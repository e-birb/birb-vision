use birb_vision::CameraDevice;
use birb_vision_mvs::{device::TransportLayerType, MVContext};
use egui::{FontData, FontDefinitions, FontFamily, Window};
use egui_interface::Preview;


struct MyApp {
    preview: Preview,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert("Material".into(), FontData::from_static(include_bytes!("../../../MaterialIcons-Regular (1).ttf")));
            fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(2, "Material".into());
            let fs = fonts.families.get_mut(&FontFamily::Proportional).unwrap();
            ctx.set_fonts(fonts);

            ui.heading("Hello World! \u{e1c4}");
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

        CameraDevice::open(&camera, Default::default()).unwrap();
        camera.open_params_gui().unwrap();
        CameraDevice::close(&camera).unwrap();

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

    println!("\u{e1c4}");
    println!("\u{e88a}");
}