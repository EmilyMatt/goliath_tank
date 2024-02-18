use super::{ApplicationState, ApplicationStateTrait, InitState, PendingState};
use crate::config::ApplicationConfig;
use eframe::egui::{Align2, Color32, FontFamily, FontId, Rect, Ui};
use goliath_common::core::NodeType;
use goliath_common::security::{generate_registration_hash, RegistrationRequest};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, error::TrySendError},
};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct NewlyConnected {
    ws_conn: (mpsc::Sender<Message>, mpsc::Receiver<Message>),
    registration_time: Option<Instant>,
}

impl NewlyConnected {
    pub fn new(ws_conn: (mpsc::Sender<Message>, mpsc::Receiver<Message>)) -> Self {
        Self {
            ws_conn,
            registration_time: None,
        }
    }

    fn register_with_backend(
        &mut self,
        rt: &Runtime,
        config: &ApplicationConfig,
    ) -> Option<ApplicationState> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went like 50 years backwards")
            .as_millis();

        if let Ok(hash) = generate_registration_hash(&config.client_id, timestamp, &config.key)
            .map_err(|err| log::error!("Could not decode key: {err}"))
        {
            if let Err(TrySendError::Closed(err)) = self.ws_conn.0.try_send(Message::Text(
                serde_json::to_string(&RegistrationRequest {
                    id: config.client_id.clone(),
                    timestamp,
                    hash,
                    node_type: NodeType::Client,
                })
                .expect("Could not serialize registration message"),
            )) {
                log::error!("Lost socket connection: {err}");
                return Some(ApplicationState::Pending(PendingState::new(
                    ApplicationState::Init(InitState::new()),
                    "Lost connection to server".to_string(),
                    rt,
                    Duration::from_secs(2),
                )));
            }
        }

        // Register even if decode failed so we don't loop out
        // TODO: add an `Exiting` application state
        self.registration_time = Some(Instant::now());

        None
    }
}

impl ApplicationStateTrait for NewlyConnected {
    fn update(&mut self, rt: &Runtime, config: &ApplicationConfig) -> Option<ApplicationState> {
        match self.registration_time.as_ref() {
            None => self.register_with_backend(rt, config),
            Some(timestamp) => {
                loop {
                    match self.ws_conn.1.try_recv() {
                        Ok(Message::Text(msg)) => {
                            if let Ok(msg) = serde_json::from_str(&msg) {
                                break;
                            };
                        }
                        Err(TryRecvError::Disconnected) => {
                            return Some(ApplicationState::Pending(PendingState::new(
                                ApplicationState::Init(InitState::new()),
                                "Lost connection to server".to_string(),
                                rt,
                                Duration::from_secs(2),
                            )));
                        }
                        Err(TryRecvError::Empty) => {
                            break;
                        }
                        _ => {}
                    }
                }

                if timestamp.elapsed().as_millis() > 2000 {
                    self.registration_time = None;
                }
                None
            }
        }
    }

    fn draw(&self, ui: &mut Ui, current_rect: Rect) {
        ui.painter().text(
            current_rect.min,
            Align2::LEFT_TOP,
            "Registering Client",
            FontId::new(24.0, FontFamily::Name("main".into())),
            Color32::from_rgb(200, 192, 5),
        );
    }
}
