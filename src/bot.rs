use crate::config::{Config, SessionStorage};
use crate::matrix::listener::MatrixListener;
use crate::matrix::responder::MatrixResponder;
use crate::webhook::listener::WebhookListener;
use tokio::sync::mpsc;
use tracing::{error, info, trace};

pub async fn init() -> anyhow::Result<()> {
    // Load config data
    let config = Config::load_config()?;

    // Matrix initalization and login
    let mut session_storage = SessionStorage::load_storage()?;
    let matrix_listener_client = ruma::client::Client::builder()
        .homeserver_url(config.mx_url.to_string())
        .access_token(session_storage.access_token)
        .build()
        .await?;

    let login_response = &matrix_listener_client
        .log_in(config.mx_uname.localpart(), &config.mx_pass, None, None)
        .await?;

    // Save returned session
    trace!("Session retrived, saving session data...");
    session_storage.access_token = Some(login_response.access_token.clone());
    if let Err(e) = session_storage.save_storage() {
        error!("{}", e);
    };
    info!("Successfully logged in as {}", config.mx_uname);

    // Clone required clients/servers and channels
    let matrix_responder_client = matrix_listener_client.clone();
    let (matrix_tx, matrix_rx) = mpsc::channel(8);
    let webhook_tx = matrix_tx.clone();

    // Create thread structures
    let mut matrix_listener = MatrixListener::new(&config, matrix_tx)?;
    let mut matrix_responder = MatrixResponder::new(matrix_rx)?;
    let webhook_listener = WebhookListener::new(&config, webhook_tx);

    // Spawn threads from thread structures, save their cached data when they exit
    let matrix_listener_task = tokio::spawn(async move {
        matrix_listener.start(matrix_listener_client).await;
        if let Err(e) = matrix_listener.storage.save_storage() {
            error!("Unable to save matrix_listener.ron on shutdown. {}", e)
        };
    });
    let webhook_listener_task = tokio::spawn(async move {
        webhook_listener.start().await;
    });
    let matrix_responder_task = tokio::spawn(async move {
        matrix_responder.start(matrix_responder_client).await;
        if let Err(e) = matrix_responder.storage.save_storage() {
            error!("Unable to save matrix_responder.ron on shutdown. {}", e)
        };
    });

    // TODO: collect errors instead of expect, and initiate clean shutdown of remaining threads on crash of a thread
    // Join threads to main thread
    matrix_listener_task
        .await
        .expect("The matrix listener task has panicked!");
    webhook_listener_task
        .await
        .expect("The webhook listener task has panicked!");
    matrix_responder_task
        .await
        .expect("The matrix responder task has panicked!");
    Ok(())
}
