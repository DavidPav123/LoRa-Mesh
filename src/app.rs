use serialport::{self, SerialPort};
use std::sync::{Arc, Mutex};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    //Init variables for the app
    #[serde(skip)]
    label: String,
    #[serde(skip)]
    shared_messages: Arc<Mutex<Vec<String>>>,
    #[serde(skip)]
    port: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
    #[serde(skip)]
    userid: Arc<Mutex<Option<String>>>,
    #[serde(skip)]
    target_user: Arc<Mutex<Option<String>>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            label: String::new(),
            shared_messages: Arc::new(Mutex::new(Vec::new())),
            port: Arc::new(Mutex::new(None)),
            userid: Arc::new(Mutex::new(None)),
            target_user: Arc::new(Mutex::new(None)),
        }
    }
}

impl TemplateApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        shared_messages: Arc<Mutex<Vec<String>>>,
        serial_port: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
        userid: Arc<Mutex<Option<String>>>,
        target_user: Arc<Mutex<Option<String>>>,
    ) -> Self {
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Update the app with the shared messages after loading
            app.shared_messages = shared_messages.clone();
            app.port = serial_port.clone();
            app.userid = userid.clone();
            app.target_user = target_user.clone();

            return app;
        }

        Default::default()
    }

    fn send_message(&self, input: &str) {
        let recipient_binding = self.target_user.lock().unwrap();
        let recipient = recipient_binding.as_ref().unwrap();
        let sender_binding = self.userid.lock().unwrap();
        let sender = sender_binding.as_ref().unwrap();
        let command = format!(
            "AT+SEND=0,{},{}{}{}\r\n",
            input.trim().len() + 48,
            recipient,
            sender,
            input.trim()
        );

        match self.port.lock() {
            Ok(mut port_option) => {
                if let Some(port) = port_option.as_mut() {
                    if let Err(_) = port.write(command.as_bytes()) {
                        println!("Error writing to port");
                    }
                }
            }
            Err(_) => {
                println!("Error locking the port");
            }
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
                    match self.shared_messages.lock() {
                        Ok(mut messages) => {
                            messages.push(format!("Message Sent: {}", self.label.clone()));
                            self.send_message(&self.label.clone());
                        }
                        Err(_) => {
                            eprintln!("Failed to lock shared_messages for writing");
                        }
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Check for new data from the serial port
            ui.vertical_centered(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Example long content to demonstrate scrolling
                    match self.shared_messages.lock() {
                        Ok(messages) => {
                            for i in messages.iter() {
                                if i.starts_with("Message Received") {
                                    let (prefix, rest) = i.split_at("Message Received: ".len());
                                    let message = rest.split_at(48).1;
                                    let full_message = format!("{}{}", prefix, message);
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}", full_message));
                                        // This spacer pushes everything to the left, showing the scroll area's full width
                                        ui.add_space(ui.available_width());
                                    });
                                } else if i.starts_with("Message Sent") {
                                    ui.horizontal(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Max),
                                            |ui| {
                                                ui.label(format!("{}", i));
                                            },
                                        );
                                    });
                                }
                            }
                        }
                        Err(_) => {
                            eprintln!("Failed to lock shared_messages for reading");
                        }
                    }
                });
            });
        });
        ctx.request_repaint()
    }
}
