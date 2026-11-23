use crate::{
    capture,
    config::AppConfig,
    input,
    vision::{self, Match, Template},
    window::{self, GameWindow},
};
use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};
use image::RgbaImage;
use std::sync::mpsc::{self, Receiver};

enum Work {
    Captured(Result<RgbaImage, String>),
    Found(Result<Option<Match>, String>),
    ClickVerified(Result<ClickVerification, String>),
}

struct ClickVerification {
    image: RgbaImage,
    hit: Match,
    point: (i32, i32),
    clicked: bool,
}
pub struct UmaPilotApp {
    config: AppConfig,
    windows: Vec<GameWindow>,
    selected_window: Option<usize>,
    templates: Vec<Template>,
    selected_template: Option<usize>,
    screenshot: Option<RgbaImage>,
    texture: Option<TextureHandle>,
    hit: Option<Match>,
    status: String,
    logs: Vec<String>,
    real_click: bool,
    confirm_click: bool,
    worker: Option<Receiver<Work>>,
}
impl UmaPilotApp {
    pub fn new(_: &eframe::CreationContext<'_>, config: AppConfig) -> Self {
        window::enable_dpi_awareness();
        let mut app = Self {
            config,
            windows: vec![],
            selected_window: None,
            templates: vec![],
            selected_template: None,
            screenshot: None,
            texture: None,
            hit: None,
            status: "Ready. Select a window and capture it.".into(),
            logs: vec![],
            real_click: false,
            confirm_click: false,
            worker: None,
        };
        app.refresh_windows();
        app.load_templates();
        app
    }
    fn log(&mut self, text: impl Into<String>) {
        let value = text.into();
        tracing::info!("{value}");
        self.logs.push(value);
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
    fn refresh_windows(&mut self) {
        match window::windows() {
            Ok(items) => {
                self.windows = items;
                self.selected_window = None;
                self.status = format!("Found {} visible windows.", self.windows.len());
            }
            Err(e) => self.status = e.to_string(),
        }
    }
    fn load_templates(&mut self) {
        match vision::load_templates(&self.config.template_directory) {
            Ok(items) => {
                self.templates = items;
                self.selected_template = None;
                self.status = format!("Loaded {} PNG template(s).", self.templates.len());
            }
            Err(e) => self.status = e.to_string(),
        }
    }
    fn selected_game(&self) -> Option<GameWindow> {
        self.selected_window
            .and_then(|i| self.windows.get(i))
            .cloned()
    }
    fn start_capture(&mut self) {
        let Some(game) = self.selected_game() else {
            self.status = "Select a valid window first.".into();
            return;
        };
        if self.worker.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.worker = Some(rx);
        self.status = "Capturing client area…".into();
        std::thread::spawn(move || {
            let _ = tx.send(Work::Captured(
                capture::capture(&game).map_err(|e| e.to_string()),
            ));
        });
    }
    fn start_find(&mut self) {
        let (Some(image), Some(template)) = (
            self.screenshot.clone(),
            self.selected_template
                .and_then(|i| self.templates.get(i))
                .cloned(),
        ) else {
            self.status = "Capture/load an image and select a template.".into();
            return;
        };
        if self.worker.is_some() {
            return;
        }
        let threshold = self.config.confidence_threshold;
        let (tx, rx) = mpsc::channel();
        self.worker = Some(rx);
        self.status = "Finding template…".into();
        std::thread::spawn(move || {
            let _ = tx.send(Work::Found(
                vision::find(&image, &template, threshold).map_err(|e| e.to_string()),
            ));
        });
    }
    fn start_click_verification(&mut self) {
        let (Some(game), Some(template)) = (
            self.selected_game(),
            self.selected_template
                .and_then(|index| self.templates.get(index))
                .cloned(),
        ) else {
            self.status = "Select a target window and template first.".into();
            return;
        };
        if self.worker.is_some() {
            return;
        }
        if self.real_click && !self.confirm_click {
            self.status = "Enable confirmation before sending a real click.".into();
            return;
        }
        let threshold = self.config.confidence_threshold;
        let real_click = self.real_click;
        if real_click {
            self.confirm_click = false;
        }
        let (tx, rx) = mpsc::channel();
        self.worker = Some(rx);
        self.status = "Capturing and verifying latest template match…".into();
        std::thread::spawn(move || {
            let outcome = (|| {
                let image = capture::capture(&game).map_err(|error| error.to_string())?;
                let hit = vision::find(&image, &template, threshold)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "NOT FOUND in fresh capture; click cancelled.".to_owned())?;
                let point = input::target_point(&game, &hit).map_err(|error| error.to_string())?;
                if real_click {
                    input::click_once(&game, point).map_err(|error| error.to_string())?;
                }
                Ok(ClickVerification {
                    image,
                    hit,
                    point,
                    clicked: real_click,
                })
            })();
            let _ = tx.send(Work::ClickVerified(outcome));
        });
    }
    fn open_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .pick_file()
        else {
            return;
        };
        match image::open(&path) {
            Ok(value) => {
                self.screenshot = Some(value.to_rgba8());
                self.hit = None;
                self.update_texture(ctx);
                self.status = format!("Loaded offline image {}", path.display());
                self.log(self.status.clone());
            }
            Err(e) => self.status = format!("Could not load image: {e}"),
        }
    }
    fn update_texture(&mut self, ctx: &egui::Context) {
        if let Some(image) = &self.screenshot {
            let size = [image.width() as usize, image.height() as usize];
            let color = ColorImage::from_rgba_unmultiplied(size, image.as_raw());
            self.texture = Some(ctx.load_texture("screenshot", color, TextureOptions::LINEAR));
        }
    }
    fn poll(&mut self, ctx: &egui::Context) {
        let result = self.worker.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(work) = result {
            self.worker = None;
            match work {
                Work::Captured(Ok(image)) => {
                    self.screenshot = Some(image);
                    self.hit = None;
                    self.update_texture(ctx);
                    self.status = "Capture succeeded.".into();
                    self.log("Captured selected window client area.");
                }
                Work::Captured(Err(e)) => {
                    self.status = format!("Capture failed: {e}");
                    self.log(self.status.clone());
                }
                Work::Found(Ok(Some(hit))) => {
                    self.status = format!(
                        "FOUND: {:.1}% at {}, {}",
                        hit.confidence * 100.,
                        hit.x,
                        hit.y
                    );
                    self.log(self.status.clone());
                    self.hit = Some(hit);
                }
                Work::Found(Ok(None)) => {
                    self.hit = None;
                    self.status = "NOT FOUND: no result meets confidence threshold.".into();
                    self.log(self.status.clone());
                }
                Work::Found(Err(e)) => {
                    self.status = format!("Find failed: {e}");
                    self.log(self.status.clone());
                }
                Work::ClickVerified(Ok(result)) => {
                    self.screenshot = Some(result.image);
                    self.hit = Some(result.hit.clone());
                    self.update_texture(ctx);
                    if result.clicked {
                        self.status =
                            format!("Clicked {}, {} once.", result.point.0, result.point.1);
                        self.log(format!(
                            "real click at {:?}; confidence {:.3}",
                            result.point, result.hit.confidence
                        ));
                    } else {
                        self.status = format!(
                            "DRY RUN: would click {}, {}",
                            result.point.0, result.point.1
                        );
                        self.log(format!(
                            "dry-run click at {:?}; confidence {:.3}",
                            result.point, result.hit.confidence
                        ));
                    }
                }
                Work::ClickVerified(Err(e)) => {
                    self.status = format!("Click blocked: {e}");
                    self.log(self.status.clone());
                }
            }
        }
    }
    fn save(&mut self) {
        let Some(image) = &self.screenshot else {
            self.status = "Nothing to save.".into();
            return;
        };
        let name = format!("capture_{}.png", chrono_name());
        let path = self.config.screenshot_directory.join(name);
        match image.save(&path) {
            Ok(_) => {
                self.status = format!("Saved {}", path.display());
                self.log(self.status.clone());
            }
            Err(e) => self.status = format!("Save failed: {e}"),
        }
    }
    fn test_click(&mut self) {
        self.start_click_verification();
    }
}
fn chrono_name() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "capture".into())
}
impl eframe::App for UmaPilotApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.poll(ctx);
        egui::SidePanel::left("controls")
            .min_width(280.)
            .show(ctx, |ui| {
                ui.heading("UmaPilot");
                if ui.button("Refresh Windows").clicked() {
                    self.refresh_windows()
                }
                ui.label("Target window");
                egui::ComboBox::from_id_salt("window")
                    .selected_text(
                        self.selected_window
                            .and_then(|i| self.windows.get(i))
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "Choose window…".into()),
                    )
                    .show_ui(ui, |ui| {
                        for (i, w) in self.windows.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_window, Some(i), w.to_string());
                        }
                    });
                if ui.button("Capture Screenshot").clicked() {
                    self.start_capture()
                }
                if ui.button("Open Screenshot PNG").clicked() {
                    self.open_screenshot(ctx)
                }
                ui.separator();
                if ui.button("Load Templates").clicked() {
                    self.load_templates()
                }
                ui.label("Template");
                egui::ComboBox::from_id_salt("template")
                    .selected_text(
                        self.selected_template
                            .and_then(|i| self.templates.get(i))
                            .and_then(|t| t.path.file_name())
                            .map(|x| x.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Choose template…".into()),
                    )
                    .show_ui(ui, |ui| {
                        for (i, t) in self.templates.iter().enumerate() {
                            let name = t
                                .path
                                .file_name()
                                .map(|n| n.to_string_lossy())
                                .unwrap_or_else(|| "unnamed".into());
                            ui.selectable_value(&mut self.selected_template, Some(i), name);
                        }
                    });
                ui.add(
                    egui::Slider::new(&mut self.config.confidence_threshold, 0.50..=1.0)
                        .text("Confidence"),
                );
                if ui.button("Find Template").clicked() {
                    self.start_find()
                }
                if ui.button("Save Screenshot").clicked() {
                    self.save()
                }
                ui.separator();
                ui.checkbox(&mut self.real_click, "Real Click (off = Dry Run)");
                if self.real_click {
                    ui.checkbox(&mut self.confirm_click, "I confirm one real click");
                }
                if ui.button("Test Click").clicked() {
                    self.test_click()
                };
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Screenshot preview");
            if let Some(tex) = &self.texture {
                let available = ui.available_size();
                let mut size = tex.size_vec2();
                let scale = (available.x / size.x)
                    .min((available.y - 40.) / size.y)
                    .min(1.);
                size *= scale;
                let response = ui.add(egui::Image::new(tex).fit_to_exact_size(size));
                if let Some(hit) = &self.hit {
                    let rect = egui::Rect::from_min_size(
                        response.rect.min + egui::vec2(hit.x as f32 * scale, hit.y as f32 * scale),
                        egui::vec2(hit.width as f32 * scale, hit.height as f32 * scale),
                    );
                    ui.painter().rect_stroke(
                        rect,
                        0.,
                        egui::Stroke::new(2.0_f32, Color32::LIGHT_GREEN),
                        egui::StrokeKind::Outside,
                    );
                }
            } else {
                ui.label("No screenshot captured yet.");
            }
        });
        egui::TopBottomPanel::bottom("status")
            .resizable(true)
            .default_height(145.)
            .show(ctx, |ui| {
                ui.colored_label(
                    if self.status.contains("failed") || self.status.contains("NOT FOUND") {
                        Color32::LIGHT_RED
                    } else {
                        Color32::LIGHT_GREEN
                    },
                    &self.status,
                );
                if let Some(hit) = &self.hit {
                    ui.label(format!(
                        "Match: x={}, y={}, w={}, h={}, confidence={:.2}%",
                        hit.x,
                        hit.y,
                        hit.width,
                        hit.height,
                        hit.confidence * 100.
                    ));
                }
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for line in self.logs.iter().rev().take(12) {
                        ui.monospace(line);
                    }
                });
            });
        if self.worker.is_some() {
            ctx.request_repaint();
        }
    }
}
