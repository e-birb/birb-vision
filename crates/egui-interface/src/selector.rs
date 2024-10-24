use std::{sync::Arc, thread::JoinHandle};

use birb_vision::core::backend::{BackendRegistry, BackendSet, DeviceInfo};
use egui::{CollapsingHeader, Grid, Label, ScrollArea};
use material_icons::Icon;
use scope_guard::scope_guard;

use crate::{add_fonts, CameraControl};


pub struct CameraManager {
    camera_control: CameraControl,

    enum_task: Option<JoinHandle<Vec<(String, DeviceInfo)>>>,
    cameras: Vec<(String, DeviceInfo)>,

    backend_registry: BackendRegistry,
}

impl CameraManager {
    pub fn new() -> Self {
        let backend_registry = birb_vision::all_backends();
        CameraManager {
            camera_control: CameraControl::new(),
            enum_task: None,
            cameras: Vec::new(),
            backend_registry,
        }
    }

    pub fn current_camera_info(&self) -> Option<Arc<DeviceInfo>> {
        self.camera_control.current_camera_info()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        add_fonts(ui.ctx());

        if let Some(task) = self.enum_task.as_ref() {
            if task.is_finished() {
                let cameras = match self.enum_task.take().unwrap().join() {
                    Ok(cameras) => cameras,
                    Err(e) => {
                        log::error!("Error enumerating cameras: {e:?}"); // TODO try downcast to String and &str
                        Vec::new()
                    }
                };
                self.cameras = cameras;
            }
        }

        ui.add_enabled_ui(!self.is_any_task_running(), |ui| {
            ui.centered_and_justified(|ui| {
                ui.horizontal(|ui| {
                    self.show_selection(ui);
                    self.camera_control.show(ui);
                });
            });
        });
    }

    pub fn show_selection(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.set_width(300.0);
            ui.horizontal(|ui| {
                if ui.button("Enumerate").clicked() {
                    self.start_enumerate(ui.ctx());
                }
                if self.enum_task.is_some() {
                    ui.spinner();
                }
                if ui.button(Icon::Close.to_string())
                    .on_hover_text("Close current device")
                    .clicked() {
                    self.camera_control = CameraControl::new();
                }
            });

            ui.separator();
            ui.centered_and_justified(|ui| { // TODO does not work
                ScrollArea::vertical().show(ui, |ui| {
                    let mut to_open = None;
                    for (backend_id, info) in &self.cameras {
                        CollapsingHeader::new(&info.display_name).id_source(&info).show(ui, |ui| {
                            if ui
                                .button("Connect \u{e63c}") // plug
                                .on_hover_text("Connect")
                                .clicked() {
                                to_open = Some((backend_id.clone(), info.clone()));
                            }
                            ui.label(backend_id);
                            Grid::new(&info).max_col_width(150.0).striped(true).show(ui, |ui| {
                                let keys = info.other.keys().collect::<Vec<_>>();
                                //keys.sort();
                                for k in keys {
                                    let v = &info.other[k];
                                    if !v.visible {
                                        continue;
                                    }
                                    ui.label(&v.display_name);
                                    ui.add(Label::new(&v.value).truncate()).on_hover_ui(|ui| {
                                        ui.style_mut().interaction.selectable_labels = true;
                                        ui.label(k);
                                    });
                                    ui.end_row();
                                }
                            });
                        });
                    }
                    if let Some((backend_id, info)) = to_open {
                        self.open_camera(&backend_id, info);
                    }
                });
            });
        });
    }

    fn is_any_task_running(&self) -> bool {
        self.enum_task.is_some()
    }

    fn start_enumerate(&mut self, cx: &egui::Context) {
        let cx = cx.clone();
        let registry = self.backend_registry.clone();
        let task = std::thread::spawn(move || {
            scope_guard!(|| {
                cx.request_repaint();
            });

            let keys = registry.all_packages().into_iter().map(|(id, _)| id).collect::<Vec<_>>();
            let backends = BackendSet::new_with_registry(registry);
            let mut all_devices = Vec::new();
            for key in keys {
                let backend = match backends.get_backend(&key).unwrap() {
                    Ok(backend) => backend,
                    Err(e) => {
                        log::error!("Error getting backend: {e}");
                        continue;
                    }
                };
                let devices = match backend.enumerate(&backend.default_transport_layers()) {
                    Ok(devices) => devices,
                    Err(e) => {
                        log::error!("Error enumerating devices: {e}");
                        continue;
                    }
                };
                all_devices.extend(devices.into_iter().map(|d| (key.clone(), d)));
            }
            all_devices
        });
        self.enum_task = Some(task);
    }

    fn open_camera(&mut self, backend_id: &str, device_info: DeviceInfo) {
        let backend_id = backend_id.to_string();
        self.camera_control = CameraControl::new(); // Close current camera
        let mut preview = CameraControl::new();
        let registry = self.backend_registry.clone();
        preview.init(move || {
            let backend = match registry.get_backend(backend_id).unwrap() {
                Ok(backend) => backend,
                Err(e) => {
                    log::error!("Error getting backend: {e}");
                    panic!("Error getting backend: {e}");
                }
            };
            let camera = match backend.create(&device_info) {
                Ok(camera) => camera,
                Err(e) => {
                    log::error!("Error opening camera: {e}");
                    panic!("Error opening camera: {e}");
                }
            };
            let Some(camera) = camera else {
                log::error!("Camera not found");
                panic!("Camera not found");
            };
            camera
        });
        // TODO use self.preview.init(...);
        self.camera_control = preview;
    }
}