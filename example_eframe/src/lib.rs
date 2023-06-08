mod app;

pub use app::DemoApp;

// ----------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    eframe::start_web(
        canvas_id,
        Default::default(),
        Box::new(|_cc| Box::new(DemoApp::default())),
    )
    .await?;
    Ok(())
}
