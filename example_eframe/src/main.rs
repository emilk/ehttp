fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    eframe::run_native(
        "ehttp demo",
        Default::default(),
        Box::new(|_cc| Box::<example_eframe::DemoApp>::default()),
    )
}
