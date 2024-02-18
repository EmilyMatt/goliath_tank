use super::{ApplicationState, ApplicationStateTrait, NewlyConnected, PendingState};
use crate::config::ApplicationConfig;
use eframe::egui::{Align2, Color32, FontFamily, FontId, Rect, Ui};
use goliath_common::websocket::{goliath_ws_connect, ConnectResult};
use std::time::Duration;
use tokio::{runtime::Runtime, task::JoinHandle};

#[derive(Debug)]
pub struct InitState {
    connection_request: Option<JoinHandle<ConnectResult>>,
}

impl InitState {
    pub fn new() -> Self {
        Self {
            connection_request: None,
        }
    }
}

impl ApplicationStateTrait for InitState {
    fn update(&mut self, rt: &Runtime, config: &ApplicationConfig) -> Option<ApplicationState> {
        if let Some(join_handle) = self.connection_request.take() {
            if join_handle.is_finished() {
                let res = rt
                    .block_on(join_handle)
                    .map_err(|err| log::error!("Could not join connection task: {err}"))
                    .and_then(|join_handle_result| join_handle_result);

                return Some(if let Ok(conn) = res {
                    ApplicationState::NewlyConnected(NewlyConnected::new(conn))
                } else {
                    ApplicationState::Pending(PendingState::new(
                        ApplicationState::Init(InitState::new()),
                        "Unable to connect, retrying".to_string(),
                        rt,
                        Duration::from_secs(2),
                    ))
                });
            } else {
                self.connection_request = Some(join_handle); // Wait some more
            }
        } else {
            self.connection_request = Some(rt.spawn({
                let ws_address = config.ws_address.clone();
                async move { goliath_ws_connect(format!("wss://{ws_address}")).await }
            }))
        }

        None
    }

    fn draw(&self, ui: &mut Ui, current_rect: Rect) {
        ui.painter().text(
            current_rect.min,
            Align2::LEFT_TOP,
            "Connecting to server...",
            FontId::new(24.0, FontFamily::Name("main".into())),
            Color32::from_rgb(200, 22, 5),
        );
    }
}
