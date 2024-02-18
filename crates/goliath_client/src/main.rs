use application_state::{ApplicationState, ApplicationStateTrait, InitState};
use config::ApplicationConfig;
use eframe::{egui, glow, App, Frame, NativeOptions, Renderer, Theme};
use goliath_common::logging::setup_logger;
use std::sync::Arc;

mod application_state;
mod config;
mod types;
mod utils;

struct GoliathClientApp {
    config: ApplicationConfig,
    rt: tokio::runtime::Runtime,
    application_state: ApplicationState,
}

impl GoliathClientApp {
    fn new(_gl: &Arc<glow::Context>) -> Self {
        // Do stuff with glow context here

        let ws_address = "localhost:8555".to_string();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Could not initialize tokio runtime");
        Self {
            application_state: ApplicationState::Init(InitState::new()),
            rt,
            config: ApplicationConfig {
                ws_address,
                client_id: "EmilyClient".to_string(),
                key: "EmilyClientSecret".to_string(),
            },
        }
    }
}

impl App for GoliathClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        ctx.request_repaint();

        if let Some(new_state) = self.application_state.update(&self.rt, &self.config) {
            log::debug!("Setting new application state to {new_state:?}");
            self.application_state = new_state;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let current_rect = match ctx.input(|i| i.viewport().inner_rect) {
                Some(res) => res,
                None => return, // Allow for graceful shutdown
            };

            self.application_state.draw(ui, current_rect);
        });
    }
}

fn main() -> eframe::Result<()> {
    setup_logger();

    eframe::run_native(
        "Goliath Tank Client",
        NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_min_inner_size(egui::Vec2::new(640.0, 360.0))
                .with_active(true)
                .with_drag_and_drop(false),
            vsync: true,
            renderer: Renderer::Glow,
            follow_system_theme: true,
            default_theme: Theme::Dark,
            ..Default::default()
        },
        Box::new(|cc| {
            utils::ui_utils::generate_font_cache(cc);
            Box::new(GoliathClientApp::new(
                cc.gl.as_ref().expect("No GL context available"),
            ))
        }),
    )
}
