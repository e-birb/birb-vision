use birb_vision_mvs::{device::TransportLayerType, MVContext};
use egui_interface::Preview;


struct MyApp {
    preview: Preview,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label("This is a simple eframe app.");
            self.preview.show(ui);
        });
    }
}

fn main() {
    env_logger::init();

    let mut preview = Preview::new();

    preview.init(|| {
        let camera = MVContext::new(None)
            .unwrap()
            .enumerate_devices([TransportLayerType::Usb]).unwrap()
            .into_iter().next().unwrap()
            .into_device(false)
            .unwrap();

        Box::new(camera)
    });

    let app = MyApp {
        preview,
    };

    eframe::run_native(
        "My eframe app",
        eframe::NativeOptions {
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(app))),
    ).unwrap();
}