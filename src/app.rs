use serialport::{self, SerialPort};
use std::io;
use std::time::Duration;
use std::vec;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    messages: Vec<String>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "".to_owned(),
            messages: vec!["First Message".to_string(), "Second Message".to_string()],
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
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
            // Create a scroll area that automatically takes up all available space
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Example long content to demonstrate scrolling
                for i in &self.messages {
                    ui.horizontal(|ui| {
                        ui.label(format!("Message: {}", i));
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
                    }
                });
            });
        });
    }
}

fn send_message(port: &mut dyn SerialPort, input: &str) {
    let length = input.trim().len();
    let command = format!("AT+SEND=0,{},{}\r\n", length, input.trim());
    port.write(command.as_bytes())
        .expect("Failed to write to serial port");
    println!("Command sent: {}", command);
}

fn receive_data(port: &mut dyn SerialPort) {
    let mut serial_buf: Vec<u8> = vec![0; 240];
    match port.read(serial_buf.as_mut_slice()) {
        Ok(t) => {
            let received_str = String::from_utf8_lossy(&serial_buf[..t]);
            if let Some(start) = received_str.find("+RCV=") {
                let data_parts: Vec<&str> = received_str[start..].split(',').collect();
                if data_parts.len() > 2 {
                    println!("Message Received: {}", data_parts[2]);
                }
            }
        }
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        Err(_e) => println!("Error reading from serial port."),
    }
}

fn open_serial_port(port_name: &str, baud_rate: u32) -> serialport::Result<Box<dyn SerialPort>> {
    serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
}
