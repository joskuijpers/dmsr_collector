use std::time::Duration;
use crate::parser::Parser;
use crate::port::{FilePort, USBPort};

mod port;
mod parser;
mod data_frame;

fn main() {
    // Open port
    let port = USBPort::new("/dev/ttyUSB0");

    // Fake port
    // let port = FilePort::new(include_str!("../../tests/testdata").as_bytes());

    // Keep reading data from the port
    for line in port {
        match line {
            // Handle some data
            Ok(raw_frame) => {
                // println!("RAW: {:?}", raw_frame);

                // Parse
                let data_frame = Parser::parse(raw_frame).unwrap();

                // Handle
                // println!("DATA: {:?}", data_frame);

                println!("[{:?}]: {:?} kW", data_frame.time(), data_frame.electricity_delivering());
            },

            // Wait for more data
            Err(port::Error::NoData) => {},

            // An actual error.
            Err(e) => println!("ERROR {:?}", e),
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}
