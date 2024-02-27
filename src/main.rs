use serialport::{self, SerialPort};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn send_command(port: &mut dyn SerialPort, input: &str) {
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

fn main() {
    let port_name = "COM8";
    let baud_rate = 9600;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            tx.send(input).expect("Failed to send data to the main thread");
        }
    });
    
    loop {
        if let Ok(input) = rx.try_recv() {
            send_command(&mut *port, &input);
        }

        receive_data(&mut *port);

        thread::sleep(Duration::from_millis(250));
    }
}
