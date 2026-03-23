use eframe::wasm_bindgen::{self, prelude::*};

use crate::DemoApp;

/// Call this once from JavaScript to start your app.
#[wasm_bindgen]
pub async fn start(canvas: web_sys::HtmlCanvasElement) -> Result<(), wasm_bindgen::JsValue> {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(|_cc| Ok(Box::<DemoApp>::default())),
        )
        .await
}
