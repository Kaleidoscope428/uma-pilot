mod app;
mod capture;
mod config;
mod input;
mod logging;
mod vision;
mod window;

fn main() -> eframe::Result<()> {
    logging::init();
    let config = config::AppConfig::load_or_create().unwrap_or_else(|error| {
        tracing::warn!(%error, "using default configuration");
        config::AppConfig::default()
    });
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "UmaPilot",
        options,
        Box::new(|cc| Ok(Box::new(app::UmaPilotApp::new(cc, config)))),
    )
}
