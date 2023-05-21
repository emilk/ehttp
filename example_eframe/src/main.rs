fn main() -> eframe::Result<()> {
    eframe::run_native(
        "ehttp demo",
        Default::default(),
        Box::new(|_cc| Box::new(example_eframe::DemoApp::default())),
    )
}
