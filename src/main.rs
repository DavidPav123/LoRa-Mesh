#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use serialport::{self, available_ports, SerialPort};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let serial_port = Arc::new(Mutex::new(open_serial_port()));
    let ownable_serial_port = serial_port.clone();

    let shared_messages = Arc::new(Mutex::new(Vec::new()));
    let messages_for_thread = shared_messages.clone();

    // Start the background thread for reading serial data
    thread::spawn(move || {
        loop {
            // Simulate reading data and append it to the shared structure
            let mut serial_buf: Vec<u8> = vec![0; 240];

            match ownable_serial_port.lock() {
                Ok(mut lock) => {
                    if let Some(port) = lock.as_mut() {
                        match port.read(serial_buf.as_mut_slice()) {
                            Ok(t) => {
                                let received_str = String::from_utf8_lossy(&serial_buf[..t]);
                                if let Some(start) = received_str.find("+RCV=") {
                                    let data_parts: Vec<&str> = received_str[start..].split(',').collect();
                                    if data_parts.len() > 2 {
                                        if let Ok(mut messages) = messages_for_thread.lock() {
                                            messages.push(format!("Message Received: {}", data_parts[2].to_string()));
                                        } else {
                                            eprintln!("Failed to lock messages_for_thread");
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to read from serial port: {}", e);
                            }
                        }
                    } else {
                        eprintln!("Serial port is not available");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to lock ownable_serial_port: {}", e);
                }
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

fn open_serial_port() -> Option<Box<dyn SerialPort>> {
    let mut port_name = String::new();

    match available_ports() {
        Ok(ports) => {
            for p in ports {
                match p.port_type {
                    serialport::SerialPortType::UsbPort(USB_info) => {
                        // Many Arduinos have a VID of 0x2341 and PIDs of 0x0042 or 0x0043
                        if USB_info.vid == 0x2341 && (USB_info.pid == 0x0042 || USB_info.pid == 0x0043) {
                            port_name = p.port_name;
                        }
                    },
                    serialport::SerialPortType::PciPort => {
                        eprintln!("Haven't implimented handling Pci Devices")
                    },
                    serialport::SerialPortType::BluetoothPort => {
                        eprintln!("Haven't implimented handling Bluetooth Devices")
                    },
                    serialport::SerialPortType::Unknown => {
                        eprintln!("Haven't implimented handling unknown devices")
                    },
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Error occurred when attempting to read available serial ports: {:?}",
                e
            );
        }
    }

    if port_name.is_empty() {
        return None;
    }

    match serialport::new(port_name, 9600)
        .timeout(Duration::from_millis(10))
        .open()
    {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("Failed to open serial port: {}", e);
            None
        }
    }
}
