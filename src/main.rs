use serialport;
use std::io::{self, Read};
use std::time::Duration;

fn main() -> serialport::Result<()> {
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
            Ok(t) => {
                let str = String::from_utf8_lossy(&serial_buf);
                        // Split the received string into lines and filter for those starting with +RCV
                        str.lines()
                            .filter(|line| line.starts_with("+RCV"))
                            .for_each(|line| println!("{}", line));
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => println!("Found no Data.", ),
        }
    }
}
