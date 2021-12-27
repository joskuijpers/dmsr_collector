use std::time::Duration;
use crate::parser::Parser;
use crate::port::FilePort;

mod port;
mod parser;
mod data_frame;

fn main() {
    // Open port
    // let port = USBPort::new("/dev/ttyUSB0")?;

    // Fake port
    let port = FilePort::new(include_str!("../../tests/testdata").as_bytes());

    let parser = Parser::new();

    // Keep reading data from the port
    for line in port {
        match line {
            // Handle some data
            Ok(raw_frame) => {
                println!("RAW: {:?}", raw_frame);

                // Parse
                let data_frame = parser.parse(raw_frame);

                // Handle
                println!("DATA: {:?}", data_frame);
            },

            // Wait for more data
            Err(port::Error::NoData) => {},

            // An actual error.
            Err(e) => println!("ERROR {:?}", e),
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}
