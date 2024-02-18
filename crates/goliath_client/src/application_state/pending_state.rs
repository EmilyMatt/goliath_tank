use crate::application_state::{ApplicationState, ApplicationStateTrait};
use crate::config::ApplicationConfig;
use eframe::egui::{Align2, Color32, FontFamily, FontId, Rect, Ui};
use std::{mem, time::Duration};
use tokio::{runtime::Runtime, task::JoinHandle};

#[derive(Debug)]
pub struct PendingState {
    next_state: Box<ApplicationState>,
    message_to_print: String,
    join_handle: JoinHandle<()>,
}

impl PendingState {
    pub fn new(
        next_state: ApplicationState,
        message_to_print: String,
        rt: &Runtime,
        time_to_wait: Duration,
    ) -> Self {
        Self {
            next_state: next_state.into(),
            message_to_print,
            join_handle: rt.spawn(async move { tokio::time::sleep(time_to_wait).await }),
        }
    }
}

impl ApplicationStateTrait for PendingState {
    fn update(&mut self, _rt: &Runtime, _config: &ApplicationConfig) -> Option<ApplicationState> {
        self.join_handle
            .is_finished()
            .then(|| *mem::replace(&mut self.next_state, ApplicationState::Dummy.into()))
        // Don't use then_some, is eagerly evaluated, dangerous
    }

    fn draw(&self, ui: &mut Ui, current_rect: Rect) {
        ui.painter().text(
            current_rect.min,
            Align2::LEFT_TOP,
            &self.message_to_print,
            FontId::new(24.0, FontFamily::Name("main".into())),
            Color32::from_rgb(200, 22, 5),
        );
    }
}
