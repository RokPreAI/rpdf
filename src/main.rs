use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("rpdf")
            .with_inner_size([1200.0, 820.0])
            .with_min_inner_size([900.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "rpdf",
        options,
        Box::new(|_cc| Ok(Box::new(rpdf::app::RpdfApp::default()))),
    )
}
