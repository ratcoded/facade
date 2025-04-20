// i don't know what im doin here man

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
}