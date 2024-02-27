use serialport::{self, SerialPort};
use std::error::Error;
use std::io;
use std::sync::{Arc, Mutex};
use std::vec;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    messages: Vec<String>,
    #[serde(skip)]
    shared_messages: Arc<Mutex<Vec<String>>>,
    #[serde(skip)]
    serial_port: Option<Arc<Mutex<Box<dyn SerialPort>>>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            label: String::new(),
            messages: Vec::new(),
            serial_port: None,
            shared_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl TemplateApp {
    fn send_message(&self, input: &str) {
        let command = format!("AT+SEND=0,{},{}\r\n", input.trim().len(), input.trim());

        // Lock the serial port for safe access
        let mut port = match self.serial_port.as_ref().expect("Something").lock() {
            Ok(port) => port,
            Err(_) => {
                eprintln!("Failed to lock the serial port");
                return;
            }
        };

        if let Err(e) = port.write(command.as_bytes()) {
            eprintln!("Failed to write to serial port: {}", e);
        }
    }

    pub fn new(
        cc: &eframe::CreationContext<'_>,
        shared_messages: Arc<Mutex<Vec<String>>>,
        serial_port: Box<dyn SerialPort>,
    ) -> Self {
        let serial_port = Arc::new(Mutex::new(serial_port));
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Update the app with the shared messages after loading
            app.shared_messages = shared_messages;
            app.serial_port = Some(serial_port);

            return app;
        }

        Default::default()
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

        egui::CentralPanel::default().show(ctx, |ui| {
            // Check for new data from the serial port

            // Create a scroll area that automatically takes up all available space
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Example long content to demonstrate scrolling
                for i in &self.messages {
                    ui.horizontal(|ui| {
                        ui.label(format!("Message Sent: {}", i));
                        // This spacer pushes everything to the left, showing the scroll area's full width
                        ui.add_space(ui.available_width());
                    });
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.label);
                    if ui.button("Send").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        self.messages.push(self.label.clone());
                        self.send_message(&self.label.clone());
                    }
                });
            });
        });
    }
}

fn receive_data(port: &mut dyn SerialPort) -> Result<String, Box<dyn Error>> {
    let mut serial_buf: Vec<u8> = vec![0; 240];
    match port.read(serial_buf.as_mut_slice()) {
        Ok(t) => {
            let received_str = String::from_utf8_lossy(&serial_buf[..t]);
            if let Some(start) = received_str.find("+RCV=") {
                let data_parts: Vec<&str> = received_str[start..].split(',').collect();
                if data_parts.len() > 2 {
                    Ok(data_parts[2].to_string())
                } else {
                    Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "No data found",
                    )))
                }
            } else {
                Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "No data found",
                )))
            }
        }
        Err(_) => Err(Box::new(serialport::Error::new(
            serialport::ErrorKind::NoDevice,
            "Couldn't read from serial port",
        ))),
    }
}
