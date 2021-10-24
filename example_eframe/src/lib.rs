use std::sync::{Arc, Mutex};

use eframe::{egui, epi};

#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Method {
    Get,
    Post,
}

enum Download {
    None,
    InProgress,
    Done(ehttp::Result<ehttp::Response>),
}

pub struct DemoApp {
    url: String,
    method: Method,
    request_body: String,
    download: Arc<Mutex<Download>>,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            url: "https://raw.githubusercontent.com/emilk/ehttp/master/README.md".to_owned(),
            method: Method::Get,
            request_body: r#"["posting some json"]"#.to_owned(),
            download: Arc::new(Mutex::new(Download::None)),
        }
    }
}

impl epi::App for DemoApp {
    fn name(&self) -> &str {
        "ehttp demo"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let trigger_fetch = self.ui_url(ui);

            if trigger_fetch {
                let request = match self.method {
                    Method::Get => ehttp::Request::get(&self.url),
                    Method::Post => {
                        ehttp::Request::post(&self.url, self.request_body.as_bytes().to_vec())
                    }
                };
                let download_store = self.download.clone();
                *download_store.lock().unwrap() = Download::InProgress;
                let repaint_signal = frame.repaint_signal();
                ehttp::fetch(request, move |response| {
                    *download_store.lock().unwrap() = Download::Done(response);
                    repaint_signal.request_repaint();
                });
            }

            ui.separator();

            let download: &Download = &self.download.lock().unwrap();
            match download {
                Download::None => {}
                Download::InProgress => {
                    ui.label("Wait for it…");
                }
                Download::Done(response) => match response {
                    Err(err) => {
                        ui.label(err);
                    }
                    Ok(response) => {
                        response_ui(ui, response);
                    }
                },
            }
        });
    }
}

impl DemoApp {
    fn ui_url(&mut self, ui: &mut egui::Ui) -> bool {
        let mut trigger_fetch = self.ui_examples(ui);

        egui::Grid::new("request_parameters")
            .spacing(egui::Vec2::splat(4.0))
            .min_col_width(70.0)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("URL:");
                trigger_fetch |= ui.text_edit_singleline(&mut self.url).lost_focus();
                ui.end_row();

                ui.label("Method:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.method, Method::Get, "GET")
                        .clicked();
                    ui.selectable_value(&mut self.method, Method::Post, "POST")
                        .clicked();
                });
                ui.end_row();

                if self.method == Method::Post {
                    ui.label("POST Body:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.request_body)
                            .code_editor()
                            .desired_rows(1),
                    );
                    ui.end_row();
                }
            });

        trigger_fetch |= ui.button("fetch ▶").clicked();

        trigger_fetch
    }

    fn ui_examples(&mut self, ui: &mut egui::Ui) -> bool {
        let mut trigger_fetch = false;

        ui.horizontal(|ui| {
            ui.label("Examples:");

            let self_url = format!(
                "https://raw.githubusercontent.com/emilk/ehttp/master/{}",
                file!()
            );
            if ui
                .selectable_label(
                    (&self.url, self.method) == (&self_url, Method::Get),
                    "GET source code",
                )
                .clicked()
            {
                self.url = self_url;
                self.method = Method::Get;
                trigger_fetch = true;
            }

            let pastebin_url = "https://httpbin.org/post".to_owned();
            if ui
                .selectable_label(
                    (&self.url, self.method) == (&pastebin_url, Method::Post),
                    "POST to httpbin.org",
                )
                .clicked()
            {
                self.url = pastebin_url;
                self.method = Method::Post;
                trigger_fetch = true;
            }
        });

        trigger_fetch
    }
}

fn response_ui(ui: &mut egui::Ui, response: &ehttp::Response) {
    ui.monospace(format!("url:          {}", response.url));
    ui.monospace(format!(
        "status:       {} ({})",
        response.status, response.status_text
    ));
    ui.monospace(format!(
        "content-type: {}",
        response.content_type().unwrap_or_default()
    ));
    ui.monospace(format!(
        "size:         {:.1} kB",
        response.bytes.len() as f32 / 1000.0
    ));

    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::CollapsingHeader::new("Response headers")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("response_headers")
                    .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                    .show(ui, |ui| {
                        for header in &response.headers {
                            ui.label(header.0);
                            ui.label(header.1);
                            ui.end_row();
                        }
                    })
            });

        ui.separator();

        if let Some(text) = response.text() {
            let tooltip = "Click to copy the response body";
            if ui.button("📋").on_hover_text(tooltip).clicked() {
                ui.output().copied_text = text.to_owned();
            }
            ui.separator();
        }

        if let Some(text) = response.text() {
            selectable_text(ui, text);
        } else {
            ui.monospace("[binary]");
        }
    });
}

fn selectable_text(ui: &mut egui::Ui, mut text: &str) {
    ui.add(
        egui::TextEdit::multiline(&mut text)
            .desired_width(f32::INFINITY)
            .text_style(egui::TextStyle::Monospace),
    );
}

// ----------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    let app = DemoApp::default();
    eframe::start_web(canvas_id, Box::new(app))
}
