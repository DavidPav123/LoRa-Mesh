use serialport::{self, SerialPort};
use std::sync::{Arc, Mutex};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    #[serde(skip)]
    shared_messages: Arc<Mutex<Vec<String>>>,
    #[serde(skip)]
    port: Arc<Mutex<Box<dyn SerialPort>>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            label: String::new(),
            shared_messages: Arc::new(Mutex::new(Vec::new())),
            //opening a random port to avoid panic
            port: Arc::new(Mutex::new(serialport::new("COM1", 9600).open().unwrap())),
            //port: Arc::new(Mutex::new(serialport::new("/dev/ttyACM1", 9600).open().unwrap())),

        }
    }
}

impl TemplateApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        shared_messages: Arc<Mutex<Vec<String>>>,
        serial_port: Arc<Mutex<Box<dyn SerialPort>>>,
    ) -> Self {
        let port = serial_port.clone();
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Update the app with the shared messages after loading
            app.shared_messages = shared_messages;
            app.port = port;

            return app;
        }

        Default::default()
    }

    fn send_message(&self, input: &str) {
        let command = format!("AT+SEND=0,{},{}\r\n", input.trim().len(), input.trim());

        // Lock the port for safe access
        let mut port = match self.port.lock() {
            Ok(port) => port,
            Err(_) => {
                eprintln!("Failed to lock the port");
                return;
            }
        };

        if let Err(e) = port.write(command.as_bytes()) {
            eprintln!("Failed to write to serial port: {}", e);
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.label);
                if ui.button("Send").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.shared_messages
                        .lock()
                        .unwrap()
                        .push(format!("Message Sent: {}",self.label.clone()));
                    self.send_message(&self.label.clone());
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Check for new data from the serial port
            ui.vertical_centered(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Example long content to demonstrate scrolling
                    for i in self.shared_messages.lock().unwrap().iter() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", i));
                            // This spacer pushes everything to the left, showing the scroll area's full width
                            ui.add_space(ui.available_width());
                        });
                    }
                });
                // Your code for the central panel goes here...
            });
            // Create a scroll area that automatically takes up all available space
        });
    }
}
