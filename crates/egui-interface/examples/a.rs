use birb_vision_core::CameraDevice;
use birb_vision_mvs::{device::TransportLayerType, MVContext};
use birb_vision_v4l::V4lDevice;
use egui::{FontData, FontDefinitions, FontFamily, Window};
use birb_vision_egui_interface::Preview;


struct MyApp {
    preview: Preview,
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
        if let Err(e) = camera.open_params_gui() {
            log::error!("Could not open the params gui: {e}");
        }
        CameraDevice::close(&camera).unwrap();

        Box::new(camera)
    });
    //preview.init(|| {
    //    let camera = V4lDevice::from_path("/dev/video0").unwrap();
    //    Box::new(camera)
    //});

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