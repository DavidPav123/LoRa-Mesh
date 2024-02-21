use serialport;
use std::io::{self, Write};
use std::sync::mpsc::{self};
use std::thread;
use std::time::Duration;

fn main() {
    let port_name = "COM8"; // Adjust this to match your system
    let baud_rate = 9600;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            tx.send(input)
                .expect("Failed to send data to the main thread");
        }
    });

    loop {
        if let Ok(input) = rx.try_recv() {
            // Calculate the length of the input minus the return and new linecharacters
            let length = input.len() - 2;

            // Construct the command string
            let command = format!("AT+SEND=0,{},{}", length, input);

            // Send the constructed command as bytes through the serial port
            port.write(command.as_bytes())
                .expect("Failed to write to serial port");
            println!("Command sent: {}", command);
        }

        // Handling received data
        let mut serial_buf: Vec<u8> = vec![0; 240];
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                let received_str = String::from_utf8_lossy(&serial_buf[..t]);
                if let Some(start) = received_str.find("+RCV=") {
                    //Debugging Received Message
                    /*print!("{}", received_str);*/
                    let data_parts: Vec<&str> = received_str[start..].split(',').collect();
                    if data_parts.len() > 2 {
                        println!("Message Received: {}", data_parts[2]); // Print the message part
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(_e) => println!("Error reading from serial port."),
        }

        // Add a small delay to prevent the loop from consuming too much CPU time
        thread::sleep(Duration::from_millis(10));
    }
}
