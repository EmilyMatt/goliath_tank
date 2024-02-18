use goliath_common::{logging::setup_logger, websocket::goliath_ws_connect};

#[tokio::main]
async fn main() -> Result<(), ()> {
    setup_logger();

    let (outgoing_tx, _outgoing_rx) =
        goliath_ws_connect("wss://localhost:8555".to_string()).await?;

    for _ in 0..100 {
        outgoing_tx
            .send(tokio_tungstenite::tungstenite::Message::Text(
                "Hello from vehicle".to_string(),
            ))
            .await
            .ok();
    }

    Ok(())
}
