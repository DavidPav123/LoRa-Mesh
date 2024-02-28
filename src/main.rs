#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use serialport::{self, SerialPort};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let port_name = "COM9"; // Example port name, adjust as needed
                            // let port_name = "/dev/ttyACM0"; // Example port name, adjust as needed
    let baud_rate = 9600;
    let serial_port = Arc::new(Mutex::new(open_serial_port(port_name, baud_rate)));
    let ownable_serial_port = serial_port.clone();

    let shared_messages = Arc::new(Mutex::new(Vec::new()));
    let messages_for_thread = shared_messages.clone();

    // Start the background thread for reading serial data
    thread::spawn(move || {
        loop {
            // Simulate reading data and append it to the shared structure
            let mut serial_buf: Vec<u8> = vec![0; 240];

            match ownable_serial_port
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .read(serial_buf.as_mut_slice())
            {
                Ok(t) => {
                    let received_str = String::from_utf8_lossy(&serial_buf[..t]);
                    if let Some(start) = received_str.find("+RCV=") {
                        let data_parts: Vec<&str> = received_str[start..].split(',').collect();
                        if data_parts.len() > 2 {
                            let mut messages = messages_for_thread.lock().unwrap();
                            messages
                                .push(format!("Message Received: {}", data_parts[2].to_string()));
                        }
                    }
                }
                Err(_) => {}
            }
            thread::sleep(Duration::from_millis(500)); // Simulate work
        }
    });

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "lora_mesh",
        native_options,
        Box::new(|cc| {
            Box::new(lora_mesh::TemplateApp::new(
                cc,
                shared_messages,
                serial_port,
            ))
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(eframe_template::TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}

fn open_serial_port(port_name: &str, baud_rate: u32) -> Option<Box<dyn SerialPort>> {
    let port = match serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to open serial port: {}", e);
            panic!("Failed to open serial port");
        }
    };
    Some(port)
}
