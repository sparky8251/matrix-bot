use crate::config::{Config, WebhookListenerConfig};
use crate::messages::MatrixMessage;
use crate::webhook_handlers::message_fn;
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

    pub async fn start(self) {
        let state = Arc::new(self);
        let app = Router::new()
            .route("/message", post(message_fn))
            .layer(Extension(state));

        axum::Server::bind(&"0.0.0.0:33333".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
