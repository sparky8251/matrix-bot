use crate::config::{Config, WebhookListenerConfig};
use crate::messages::MatrixMessage;
use crate::webhook_handlers::message_fn;
use axum::{extract::Extension, routing::post, Router};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use tracing::{error, trace};

pub struct WebhookListener {
    pub send: Sender<MatrixMessage>,
    pub config: WebhookListenerConfig,
}

impl WebhookListener {
    pub fn new(config: &Config, send: Sender<MatrixMessage>) -> Self {
        let config = WebhookListenerConfig {
            token: config.webhook_token.clone(),
        };
        WebhookListener { send, config }
    }

    pub async fn start(self, mut shutdown_rx: Receiver<bool>) {
        let state = Arc::new(self);
        let app = Router::new()
            .route("/message", post(message_fn))
            .layer(Extension(state));

        // TODO: enable customized binding of ip and port
        let server = axum::Server::bind(
            &"0.0.0.0:33333"
                .parse()
                .expect("Unable to bind 0.0.0.0:33333"),
        )
        .serve(app.into_make_service());

        tokio::select! {
                r = server => {
                if let Err(_) = r {
                    error!("Unable to start webhook listener");
                    return;
                }
            },
            _ = shutdown_rx.changed() => {
                trace!("Received shutdown on webhook listener thread, exiting thread with code 0");
                return;
            }
        }
    }
}
