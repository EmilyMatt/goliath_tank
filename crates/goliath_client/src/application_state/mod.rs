pub use init_state::InitState;
pub use newly_connected::NewlyConnected;
pub use pending_state::PendingState;

use crate::config::ApplicationConfig;
use eframe::egui::{Rect, Ui};
use tokio::runtime::Runtime;

mod init_state;
mod newly_connected;
mod pending_state;

pub trait ApplicationStateTrait {
    fn update(&mut self, rt: &Runtime, config: &ApplicationConfig) -> Option<ApplicationState>;

    fn draw(&self, ui: &mut Ui, current_rect: Rect);
}

#[derive(Debug, Default)]
pub enum ApplicationState {
    #[default]
    Dummy,
    Init(InitState),
    NewlyConnected(NewlyConnected),
    Pending(PendingState),
}

impl ApplicationStateTrait for ApplicationState {
    fn update(&mut self, rt: &Runtime, config: &ApplicationConfig) -> Option<ApplicationState> {
        match self {
            ApplicationState::Dummy => None,
            ApplicationState::Pending(pending_state) => pending_state.update(rt, config),
            ApplicationState::Init(init) => init.update(rt, config),
            ApplicationState::NewlyConnected(newly_connected) => newly_connected.update(rt, config),
        }
    }

    fn draw(&self, ui: &mut Ui, current_rect: Rect) {
        match self {
            ApplicationState::Dummy => {}
            ApplicationState::Pending(pending_state) => pending_state.draw(ui, current_rect),
            ApplicationState::Init(init) => init.draw(ui, current_rect),
            ApplicationState::NewlyConnected(newly_connected) => {
                newly_connected.draw(ui, current_rect)
            }
        }
    }
}
