use serialport;
use std::io::{self, Read, Write};
use std::time::Duration;

/*fn main() -> serialport::Result<()> {
    let port_name = "COM6"; // Adjust this to match your system
    let baud_rate = 57600;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");
    println!("Serial port opened successfully.");
    let mut serial_buf: Vec<u8> = vec![0; 32];
    println!("Reading data:");

    loop {
        match port.read(serial_buf.as_mut_slice()) {
            Ok(_t) => {
                let str = String::from_utf8_lossy(&serial_buf);
                        // Split the received string into lines and filter for those starting with +RCV
                        str.lines()
                            .filter(|line| line.starts_with("+RCV"))
                            .for_each(|line| println!("{}", line));
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(_e) => println!("Found no Data.", ),
        }
    }
}*/

fn main() {
    let port_name = "COM8"; // Adjust this to match your system
    let baud_rate = 57600;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    println!("Enter data to send:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Calculate the length of the input minus the return and new linecharacters
    let length = input.len() - 2;

    // Construct the command string
    let command = format!("AT+SEND=0,{},{}", length, input);

    // Send the constructed command as bytes through the serial port
    port.write(command.as_bytes())
        .expect("Failed to write to serial port");
    println!("Command sent: {}", command);
}
