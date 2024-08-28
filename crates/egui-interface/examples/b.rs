use birb_vision_core::CameraDevice;
use birb_vision_mvs::{device::TransportLayerType, MVContext};
use egui::{FontData, FontDefinitions, FontFamily};
use birb_vision_egui_interface::{Preview, Selector};


struct MyApp {
    selector: Selector,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert("Material".into(), FontData::from_static(include_bytes!("../../../MaterialIcons-Regular.ttf")));
            fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(2, "Material".into());
            ctx.set_fonts(fonts);

            ui.heading("Hello World! \u{e1c4}");
            ui.label("This is a simple eframe app.");
            self.selector.show(ui);
        });
    }
}

fn main() {
    env_logger::init();

    let selector = Selector::new();

    let app = MyApp {
        selector,
    };

    eframe::run_native(
        "My eframe app",
        eframe::NativeOptions {
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(app))),
    ).unwrap();

    println!("\u{e1c4}");
    println!("\u{e88a}");
}