use birb_vision::core::CameraDevice;
use birb_vision_mvs::{device::TransportLayerType, MVContext};
use egui::{CentralPanel, FontData, FontDefinitions, FontFamily};
use birb_vision_egui_interface::{add_fonts, CameraControl, CameraManager};


struct MyApp {
    selector: CameraManager,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.selector.show(ui);
        });
    }
}

fn main() {
    env_logger::init();

    let selector = CameraManager::new();

    let app = MyApp {
        selector,
    };

    eframe::run_native(
        "My eframe app",
        eframe::NativeOptions {
            window_builder: Some(Box::new(|builder| builder
                .with_resizable(true)
            )),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(app))),
    ).unwrap();

    println!("\u{e1c4}");
    println!("\u{e88a}");
}