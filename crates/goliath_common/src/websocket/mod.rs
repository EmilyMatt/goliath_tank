use crate::security::NoVerifier;
use futures_util::{StreamExt, TryStreamExt};
use rustls::ClientConfig;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite, Connector};

pub type ConnectResult = Result<
    (
        mpsc::Sender<tungstenite::Message>,
        mpsc::Receiver<tungstenite::Message>,
    ),
    (),
>;

pub async fn goliath_ws_connect(address: impl Into<String>) -> ConnectResult {
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth();

    let (stream, _) = connect_async_tls_with_config(
        address.into(),
        None,
        false,
        Some(Connector::Rustls(Arc::new(config))),
    )
    .await
    .map_err(|err| log::error!("{err}"))?;

    let (incoming_tx, incoming_rx) = mpsc::channel::<tungstenite::Message>(64);
    let (outgoing_tx, outgoing_rx) = mpsc::channel::<tungstenite::Message>(64);
    let (ws_write, mut ws_read) = stream.split();
    tokio::spawn(ReceiverStream::new(outgoing_rx).map(Ok).forward(ws_write));
    tokio::spawn(async move {
        loop {
            match ws_read.try_next().await {
                Ok(maybe_msg) => {
                    if let Some(msg) = maybe_msg {
                        incoming_tx.try_send(msg).ok();
                    }
                }
                Err(err) => {
                    log::error!("{err}");
                    break;
                }
            }
        }
    });

    Ok((outgoing_tx, incoming_rx))
}
