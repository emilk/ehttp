use std::sync::{Arc, Mutex};

use eframe::{egui, epi};

pub struct DemoApp {
    url: String,
    response: Arc<Mutex<Option<ehttp::Result<ehttp::Response>>>>,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            url: "https://raw.githubusercontent.com/emilk/ehttp/master/README.md".to_owned(),
            response: Default::default(),
        }
    }
}

impl epi::App for DemoApp {
    fn name(&self) -> &str {
        "ehttp demo"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let enter = ui.text_edit_singleline(&mut self.url).lost_focus()
                && ui.input().key_pressed(egui::Key::Enter);
            let click = ui
                .add(egui::Button::new("Fetch").enabled(!self.url.is_empty()))
                .clicked();

            if enter || click {
                let response_store = self.response.clone();
                let repaint_signal = frame.repaint_signal();
                ehttp::fetch(ehttp::Request::get(&self.url), move |response| {
                    *response_store.lock().unwrap() = Some(response);
                    repaint_signal.request_repaint();
                });
            }

            if let Some(response) = self.response.lock().unwrap().as_ref() {
                ui.separator();
                ui.monospace(format!("{:#?}", response));
            }
        });
    }
}

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
