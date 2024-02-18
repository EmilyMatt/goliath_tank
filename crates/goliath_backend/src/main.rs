use goliath_common::logging;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
    sync::Arc,
};
use tokio_rustls::{
    rustls::{
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
        ServerConfig,
    },
    TlsAcceptor,
};

pub mod cache_db;
pub mod security;
pub mod server_core;

#[tokio::main]
async fn main() -> Result<(), ()> {
    logging::setup_logger();

    let port = std::env::var("GOLIATH_SERVER_PORT").unwrap_or_else(|_| {
        log::warn!("No GOLIATH_SERVER_PORT environment variable found, defaulting to 8555");
        "8555".to_string()
    });
    log::info!("Using configuration port: {}", port);

    let (cert, key) =
        security::generate_certification_and_keys().map_err(|err| log::error!("{err}"))?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert.0)],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.0)),
        )
        .map_err(|err| log::error!("{err}"))?;
    let tls_acceptor = TlsAcceptor::from(Arc::new(config));

    let tcp_socket = tokio::net::TcpListener::bind(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        u16::from_str(&port).map_err(|err| log::error!("{err}"))?,
    ))
    .await
    .map_err(|err| log::error!("{err}"))?;
    log::info!(
        "Socket bound and listening on {}",
        tcp_socket.local_addr().expect("Unable to parse address")
    );

    let rt = tokio::runtime::Handle::current();

    let mut server_core = server_core::ServerCore::create();
    'main_loop: loop {
        if server_core.update(&rt) {
            break;
        }

        let (stream, addr) = match tcp_socket.accept().await {
            Ok(res) => res,
            Err(err) => {
                log::debug!("{err}");
                continue 'main_loop;
            }
        };

        log::trace!("Received connection from: {addr}");

        let tls_stream = match tls_acceptor.accept(stream).await {
            Ok(res) => res,
            Err(err) => {
                log::debug!("{err}");
                continue 'main_loop;
            }
        };

        log::trace!("Accepted TLS Stream");

        let ws_socket = match tokio_tungstenite::accept_async(tls_stream).await {
            Ok(res) => res,
            Err(err) => {
                log::debug!("{err}");
                continue 'main_loop;
            }
        };

        server_core.on_node_connected(ws_socket).await;
    }

    Ok(())
}
