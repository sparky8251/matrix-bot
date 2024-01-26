use crate::config::Config;
use crate::database::insert_or_update;
use crate::database::models::{AccessToken, CorrectionTimeCooldown, LastSync};
use crate::services::matrix::listener::MatrixListener;
use crate::services::matrix::responder::MatrixResponder;
use crate::services::webhook::listener::WebhookListener;
use anyhow::Context;
use native_db::{Database, DatabaseBuilder};
use std::env;
use std::path::PathBuf;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{mpsc, watch};
use tracing::{info, trace};

pub async fn init() -> anyhow::Result<()> {
    // Load config data
    let config = Config::load_config()?;

    let path = match env::var("MATRIX_BOT_DATA_DIR") {
        Ok(v) => [v, "database.nativedb".to_string()]
            .iter()
            .collect::<PathBuf>(),
        Err(_) => ["database.nativedb"].iter().collect::<PathBuf>(),
    };

    let mut builder = Box::new(DatabaseBuilder::new());
    //load models
    builder
        .define::<AccessToken>()
        .context("Unable to load access token database model")?;
    builder
        .define::<LastSync>()
        .context("Unable to load last sync database model")?;
    builder
        .define::<CorrectionTimeCooldown>()
        .context("Unable to load correction time cooldown database model")?;
    let static_builder: &'static DatabaseBuilder = Box::leak(builder);
    //open db
    let db = Box::new(
        static_builder
            .create(&path)
            .with_context(|| format!("Unable to create/open db {}", &path.display()))?,
    );
    let static_db: &'static Database = Box::leak(db);
    // fetch access_token

    let r = static_db
        .r_transaction()
        .context("Unable to get read transaction from db")?;
    let access_token = match r
        .get()
        .primary::<AccessToken>(1u8)
        .context("Unable to fetch access token")?
    {
        None => None,
        Some(v) => Some(v.access_token),
    };
    // drop the guard and rw transaction before await points to avoid deadlocks
    std::mem::drop(r);

    // Matrix initalization and login
    let matrix_listener_client = ruma::client::Client::builder()
        .homeserver_url(config.mx_url.to_string())
        .access_token(access_token.clone())
        .build()
        .await?;

    // Set access token as part of log_in if not set prior
    let login_response = &matrix_listener_client
        .log_in(config.mx_uname.localpart(), &config.mx_pass, None, None)
        .await?;

    // Save returned session
    let rw = static_db
        .rw_transaction()
        .context("Unable to get read transaction from db")?;
    trace!("Session retrieved, saving session data...");
    insert_or_update(
        &rw,
        AccessToken {
            id: 1,
            access_token: access_token.unwrap_or(String::new()),
        },
        AccessToken {
            id: 1,
            access_token: login_response.clone().access_token,
        },
    )?;
    // since we dont technically return from this function, explicitly drop the guard to free it for future use
    rw.commit()
        .context("Unable to commit access_token transaction")?;
    info!("Successfully logged in as {}", config.mx_uname);

    // Clone required clients/servers and channels
    let matrix_responder_client = matrix_listener_client.clone();
    let (matrix_tx, matrix_rx) = mpsc::channel(8);
    let webhook_tx = matrix_tx.clone();

    // Create thread structures
    let mut matrix_listener = MatrixListener::new(&config, matrix_tx, &static_db)?;
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
