use crate::config::{Config, SessionStorage};
use crate::matrix::listener::MatrixListener;
use crate::matrix::responder::MatrixResponder;
use crate::webhook::listener::WebhookListener;
use ruma_client::Client;
use tokio::sync::mpsc;
use tracing::{info, trace};

pub async fn init() {
    // Load config data
    let config = Config::load_config();

    // Matrix initalization and login
    let mut session_storage = SessionStorage::load_storage();
    let matrix_listener_client = Client::new(config.mx_url.clone(), session_storage.session);
    let session = &matrix_listener_client
        .log_in(config.mx_uname.localpart(), &config.mx_pass, None, None)
        .await
        .unwrap();

    // Save returned session
    trace!("Session retrived, saving session data...");
    session_storage.session = Some(session.clone());
    session_storage.save();
    info!("Successfully logged in as {}", config.mx_uname);

    // Clone required clients/servers and channels
    let matrix_responder_client = matrix_listener_client.clone();
    let (matrix_tx, matrix_rx) = mpsc::channel(8);
    let webhook_tx = matrix_tx.clone();

    // Create thread structures
    let mut matrix_listener = MatrixListener::new(&config, matrix_tx);
    let mut matrix_responder = MatrixResponder::new(matrix_rx);
    let webhook_listener = WebhookListener::new(&config, webhook_tx);

    // Spawn threads from thread structures, save their cached data when they exit
    let matrix_listener_task = tokio::spawn(async move {
        matrix_listener.start(matrix_listener_client).await;
        matrix_listener.storage.save_storage();
    });
    let matrix_responder_task = tokio::spawn(async move {
        matrix_responder.start(matrix_responder_client).await;
        matrix_responder.storage.save_storage();
    });
    let webhook_listener_task = tokio::spawn(async move {
        webhook_listener.start().await;
    });

    // Join threads to main thread
    matrix_listener_task
        .await
        .expect("The matrix listener task has panicked!");
    matrix_responder_task
        .await
        .expect("The matrix responder task has panicked!");
    webhook_listener_task
        .await
        .expect("The webhook listener task has panicked!");
}
