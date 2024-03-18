#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// hide console window on Windows in release
use lora_mesh::app::Message;
use serialport::{self, available_ports, SerialPort};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let serial_port = Arc::new(Mutex::new(open_serial_port()));
    let shared_messages: Arc<Mutex<HashMap<String, Vec<Message>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let userid = get_username(serial_port.clone());
    let target_user = Arc::new(Mutex::new(std::option::Option::Some(
        "002E0051044A7EE1000026BF".to_string(),
    )));

    if let Some(name) = userid.clone() {
        println!("{}", name);
    } else {
        println!("No userid found");
    }

    start_serial_read_thread(
        serial_port.clone(),
        shared_messages.clone(),
        userid.clone().unwrap(),
    );

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
                userid,
                target_user,
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
    let port_name = (|| -> Result<String, Box<dyn std::error::Error>> {
        let ports = available_ports()?;
        for p in ports {
            match p.port_type {
                serialport::SerialPortType::UsbPort(usb_info) => {
                    // Many Arduinos have a VID of 0x2341 and PIDs of 0x0042 or 0x0043
                    if usb_info.vid == 0x2341 && (usb_info.pid == 0x0042 || usb_info.pid == 0x0043)
                    {
                        return Ok(p.port_name);
                    }
                }
                serialport::SerialPortType::PciPort => {
                    eprintln!("Haven't implemented handling Pci Devices")
                }
                serialport::SerialPortType::BluetoothPort => {
                    eprintln!("Haven't implemented handling Bluetooth Devices")
                }
                serialport::SerialPortType::Unknown => {
                    eprintln!("Haven't implemented handling unknown devices")
                }
            }
        }
        Err("No suitable port found".into())
    })()
    .ok()?;

    serialport::new(port_name, 9600)
        .timeout(Duration::from_millis(10))
        .open()
        .map_err(|e| {
            eprintln!("Failed to open serial port: {}", e);
        })
        .ok()
}

fn start_serial_read_thread(
    ownable_serial_port: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
    messages_for_thread: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    userid: String,
) {
    thread::spawn(move || loop {
        let mut serial_buf: Vec<u8> = vec![0; 300];
        let mut received_str = String::new();

        match ownable_serial_port.lock() {
            Ok(mut lock) => {
                if let Some(port) = lock.as_mut() {
                    loop {
                        let t = port.read(&mut serial_buf);
                        match t {
                            Ok(count) => {
                                received_str
                                    .push_str(&String::from_utf8_lossy(&serial_buf[..count]));
                                if received_str.ends_with("\r\n") {
                                    break;
                                } else {
                                    thread::sleep(Duration::from_millis(25));
                                    continue;
                                }
                            }
                            Err(err) => {
                                //eprintln!("Failed to read from serial port: {}", err);
                                break;
                            }
                        }
                    }

                    if let Some(start) = received_str.find("+RCV=") {
                        let data_parts: Vec<&str> = received_str[start..].split(',').collect();
                        if received_str.contains(&userid) {
                            if let Ok(mut messages) = messages_for_thread.lock() {
                                if data_parts[2][..9].contains("CONFIRMED") {
                                    // Find the message using the senders address and mark the message as confirmed
                                    if let Some(messages_vec) =
                                        messages.get_mut(&data_parts[2][43..67].to_string())
                                    {
                                        for message in messages_vec.iter_mut() {
                                            if message.time
                                                == data_parts[2][9..19].parse().unwrap_or_default()
                                            {
                                                message.confirmed = true;
                                            }
                                        }
                                    }
                                } else {
                                    messages
                                        .entry(data_parts[2][24..48].to_string())
                                        .or_insert_with(Vec::new)
                                        .push(Message {
                                            recipient: data_parts[2][..24].to_string(),
                                            sender: data_parts[2][24..48].to_string(),
                                            time: data_parts[2][48..58].parse().unwrap_or_default(),
                                            data: data_parts[2][58..].to_string(),
                                            confirmed: true,
                                            count: 1,
                                        });

                                    send_message(
                                        "CONFIRMED".to_string(),
                                        67,
                                        data_parts[2][24..48].to_string(),
                                        data_parts[2][..24].to_string(),
                                        data_parts[2][48..58].to_string(),
                                        "".to_string(),
                                        port,
                                    );
                                }
                            }
                        } else {
                            if received_str[4..14].contains("CONFIRMED") {
                                send_message(
                                    "CONFIRMED".to_string(),
                                    data_parts[1].parse().unwrap(),
                                    data_parts[2][24..48].to_string(),
                                    data_parts[2][48..72].to_string(),
                                    data_parts[2][14..24].to_string(),
                                    "".to_string(),
                                    port,
                                );
                            } else {
                                if received_str.len() > 58 {
                                    send_message(
                                        "SEND".to_string(),
                                        data_parts[1].parse().unwrap(),
                                        data_parts[2][..24].to_string(),
                                        data_parts[2][24..48].to_string(),
                                        data_parts[2][48..58].to_string(),
                                        data_parts[2][58..].to_string(),
                                        port,
                                    );
                                }
                            }
                        }
                    }
                } else {
                    //eprintln!("Serial port is not available");
                }
            }
            Err(poisoned) => {
                eprintln!("Mutex was poisoned. Inner error: {:?}", poisoned);
            }
        }

        thread::sleep(Duration::from_millis(25));
    });
}

fn get_username(ownable_serial_port: Arc<Mutex<Option<Box<dyn SerialPort>>>>) -> Option<String> {
    (|| -> Result<String, Box<dyn std::error::Error>> {
        let mut lock = ownable_serial_port.lock()?;
        if let Some(port) = lock.as_mut() {
            thread::sleep(Duration::from_millis(2000));
            let mut serial_buf: Vec<u8> = vec![0; 240];
            port.write(format!("AT+UID?\r\n").as_bytes())?;
            thread::sleep(Duration::from_millis(100));
            let t = port.read(serial_buf.as_mut_slice())?;
            let received_str = String::from_utf8_lossy(&serial_buf[..t]);
            if let Some(start) = received_str.find("+UID=") {
                let data_parts: Vec<&str> = received_str[start..].split('=').collect();
                if data_parts.len() == 2 {
                    return Ok(data_parts[1].trim().to_string());
                }
            }
        }
        Err("Serial port is not available".into())
    })()
    .ok()
}

fn send_message(
    command_type: String,
    input_len: i32,
    recipient: String,
    sender: String,
    time_stamp: String,
    input: String,
    serial_port: &mut Box<dyn SerialPort>,
) {
    let command: String;
    if command_type == "CONFIRMED" {
        command = format!(
            "AT+SEND=0,{},CONFIRMED{}{}{}\r\n",
            input_len, time_stamp, recipient, sender
        );
    } else if command_type == "SEND" {
        command = format!(
            "AT+SEND=0,{},{}{}{}{}\r\n",
            input.trim().len() + 58,
            recipient,
            sender,
            time_stamp,
            input.trim()
        );
    } else {
        eprintln!("Invalid command type");
        return;
    }
    eprintln!("{}", command);
    if let Err(_) = serial_port.write(command.as_bytes()) {
        eprintln!("Error writing to port");
    }
}
