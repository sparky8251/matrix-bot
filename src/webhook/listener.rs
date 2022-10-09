use crate::config::{Config, WebhookListenerConfig};
use crate::messages::MatrixMessage;
use crate::webhook_handlers::message_fn;
use anyhow::Context;
use axum::{extract::Extension, routing::post, Router};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

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

    pub async fn start(self) -> anyhow::Result<()> {
        let state = Arc::new(self);
        let app = Router::new()
            .route("/message", post(message_fn))
            .layer(Extension(state));

        // TODO: enable customized binding of ip and port
        axum::Server::bind(
            &"0.0.0.0:33333"
                .parse()
                .context("Unable to parse bind address")?,
        )
        .serve(app.into_make_service())
        .await
        .context("Unable to start server")?;

        Ok(())
    }
}
