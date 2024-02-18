mod types;
mod unsorted_nodes;

use crate::cache_db::CacheDb;
use crate::server_core::types::{IdentifiedNode, IdentifiedNodeList, RegistrationTypeResponse};
use goliath_common::{core::NodeType, security::RegistrationRequest, ClientConnection};
use std::collections::HashMap;
use std::{sync::Arc, thread, time::Duration};
use tokio::{runtime::Handle, sync as TokioSync};
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::Message;

fn internal_thread(
    kill_switch_rx: TokioSync::oneshot::Receiver<()>,
    available_vehicles: IdentifiedNodeList,
    available_clients: IdentifiedNodeList,
) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("Could not construct tokio runtime");

    rt.spawn({
        async move {
            for (id, ws_conn) in available_clients.lock().await.iter_mut() {
                if let Ok(Some(Message::Text(msg))) = ws_conn.try_next().await {}
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    rt.spawn(async move {
        for (id, ws_conn) in available_vehicles.lock().await.iter_mut() {
            if let Ok(Some(Message::Text(msg))) = ws_conn.try_next().await {}
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    });
    rt.block_on(kill_switch_rx).ok();
}

async fn registration_task(
    mut ws_conn: ClientConnection,
    unsorted_nodes_tx: TokioSync::mpsc::Sender<RegistrationTypeResponse>,
) {
    if let Ok(Some(Message::Text(msg))) = ws_conn.try_next().await {
        if let Ok(registration_message) = serde_json::from_str::<RegistrationRequest>(&msg) {
            if let Ok(node_type) =
                registration_message.verify_hash(Arc::new(TokioSync::Mutex::new(CacheDb::new())))
            {
                unsorted_nodes_tx
                    .try_send(RegistrationTypeResponse {
                        id: registration_message.id,
                        node_type,
                        ws_conn,
                    })
                    .ok();
                return;
            }
        }
    }

    unsorted_nodes_tx
        .send(RegistrationTypeResponse {
            id: "".to_string(),
            node_type: NodeType::Unsorted,
            ws_conn,
        })
        .await
        .ok();
}

pub struct ServerCore {
    available_vehicles: IdentifiedNodeList,
    available_clients: IdentifiedNodeList,
    active_sessions: HashMap<String, (IdentifiedNode, IdentifiedNode)>,
    #[allow(clippy::type_complexity)]
    node_registration_channel: (
        TokioSync::mpsc::Sender<RegistrationTypeResponse>,
        TokioSync::mpsc::Receiver<RegistrationTypeResponse>,
    ),
    thread_handle: Option<(TokioSync::oneshot::Sender<()>, thread::JoinHandle<()>)>,
}

impl ServerCore {
    pub fn create() -> Self {
        let available_vehicles = Arc::new(TokioSync::Mutex::new(HashMap::new()));
        let available_clients = Arc::new(TokioSync::Mutex::new(HashMap::new()));

        let (kill_switch_tx, kill_switch_rx) = TokioSync::oneshot::channel();
        let join_handle = thread::Builder::new()
            .name("WS Handler Thread".to_string())
            .spawn({
                let available_vehicles = available_vehicles.clone();
                let available_clients = available_clients.clone();
                move || internal_thread(kill_switch_rx, available_vehicles, available_clients)
            })
            .expect("Could not launch WS Handler Thread");

        Self {
            node_registration_channel: TokioSync::mpsc::channel(128),
            available_vehicles,
            available_clients,
            active_sessions: HashMap::new(),
            thread_handle: Some((kill_switch_tx, join_handle)),
        }
    }

    pub async fn on_node_connected(&mut self, ws_socket: ClientConnection) {
        tokio::spawn(registration_task(
            ws_socket,
            self.node_registration_channel.0.clone(),
        ));
    }

    // If this function returns true, then we need to shut down the server cause shit has gone bad
    pub fn update(&mut self, rt: &Handle) -> bool {
        while let Ok(mut response) = self.node_registration_channel.1.try_recv() {
            match response.node_type {
                NodeType::Unsorted => {
                    rt.block_on(async {
                        response.ws_conn.close(None).await.ok();
                    });
                }
                NodeType::Client => {
                    self.available_clients
                        .blocking_lock()
                        .insert(response.id, response.ws_conn);
                }
                NodeType::Vehicle => {
                    self.available_vehicles
                        .blocking_lock()
                        .insert(response.id, response.ws_conn);
                }
            }
        }

        self.thread_handle
            .as_ref()
            .map(|(_, join_handle)| join_handle.is_finished())
            .unwrap_or_default()
    }
}

impl Drop for ServerCore {
    fn drop(&mut self) {
        if let Some((kill_switch_tx, join_handle)) = self.thread_handle.take() {
            kill_switch_tx.send(()).ok();
            join_handle.join().ok();
        }
    }
}
