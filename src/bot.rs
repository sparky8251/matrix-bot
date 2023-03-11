use crate::config::Config;
use crate::services::matrix::listener::MatrixListener;
use crate::services::matrix::responder::MatrixResponder;
use crate::services::webhook::listener::WebhookListener;
use anyhow::Context;
use sled::IVec;
use std::env;
use std::path::PathBuf;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{mpsc, watch};
use tracing::{info, trace};

pub async fn init() -> anyhow::Result<()> {
    // Load config data
    let config = Config::load_config()?;

    let path = match env::var("MATRIX_BOT_DATA_DIR") {
        Ok(v) => [v, "database.sled".to_string()].iter().collect::<PathBuf>(),
        Err(_) => ["database.sled"].iter().collect::<PathBuf>(),
    };

    let db = sled::open(&path).context(format!(
        "Unable to open database at {}",
        path.to_string_lossy()
    ))?;

    let session_storage = db.open_tree("session_storage")?;
    let listener_storage = db.open_tree("listener_storage")?;

    let access_token = session_storage
        .get("access_token")?
        .map(|b| String::from_utf8(b.to_vec()).unwrap()); // TODO: Try and make this cleaner error wise?

    // Matrix initalization and login
    let matrix_listener_client = ruma::client::Client::builder()
        .homeserver_url(config.mx_url.to_string())
        .access_token(access_token)
        .build()
        .await?;

    // Set access token as part of log_in if not set prior
    let login_response = &matrix_listener_client
        .log_in(config.mx_uname.localpart(), &config.mx_pass, None, None)
        .await?;

    // Save returned session
    trace!("Session retrived, saving session data...");
    let _ = session_storage.insert(
        "access_token",
        IVec::from(login_response.access_token.as_bytes()),
    );
    session_storage.flush_async().await?;
    info!("Successfully logged in as {}", config.mx_uname);

    // Clone required clients/servers and channels
    let matrix_responder_client = matrix_listener_client.clone();
    let (matrix_tx, matrix_rx) = mpsc::channel(8);
    let webhook_tx = matrix_tx.clone();

    // Create thread structures
    let mut matrix_listener = MatrixListener::new(&config, matrix_tx, listener_storage.clone())?;
    let mut matrix_responder = MatrixResponder::new(matrix_rx)?;
    let webhook_listener = WebhookListener::new(&config, webhook_tx);

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let matrix_listener_shutdown_rx = shutdown_rx.clone();
    let matrix_responder_shutdown_rx = shutdown_rx.clone();
    let webhook_listener_shutdown_rx = shutdown_rx.clone();

    // Spawn threads from thread structures, save their cached data when they exit
    let matrix_listener_task = tokio::spawn(async move {
        matrix_listener
            .start(matrix_listener_client, matrix_listener_shutdown_rx)
            .await;
        listener_storage.flush().unwrap();
    });
    let webhook_listener_task = tokio::spawn(async move {
        webhook_listener.start(webhook_listener_shutdown_rx).await;
    });
    let matrix_responder_task = tokio::spawn(async move {
        matrix_responder
            .start(matrix_responder_client, matrix_responder_shutdown_rx)
            .await;
    });

    let mut terminate = signal(SignalKind::terminate())?;
    let mut hangup = signal(SignalKind::hangup())?;

    loop {
        tokio::select! {
            _ = terminate.recv() => {
                trace!("Received SIGTERM on main thread");
                shutdown_tx.send(true)?;
                break;
            },
            _ = hangup.recv() => {
                trace!("Received SIGHUP on main thread");
            }
        };
    }

    // TODO: collect errors instead of expect, and initiate clean shutdown of remaining threads on crash of a thread
    // Join threads to main thread
    matrix_listener_task.await?;
    webhook_listener_task.await?;
    matrix_responder_task.await?;

    Ok(())
}
