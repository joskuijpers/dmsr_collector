use std::time::Duration;
use crate::parser::Parser;
use crate::port::PortBuilder;
use crate::reader::FrameReader;

mod port;
mod parser;
mod data_frame;
mod reader;

fn main() {
    // Open port
    // let port = PortBuilder::from_device("/dev/ttyUSB0");

    // Fake port
    let port = PortBuilder::from_data(include_str!("../../tests/testdata").as_bytes());
    let mut frame_reader = FrameReader::new(port);

    loop {
        if let Some(raw_frame) = frame_reader.read_next_byte() {
            let data_frame = Parser::parse(raw_frame).unwrap();

            println!("[{:?}]: {:?} kW ({:?} + {:?} kWh on meter), {:?} m3 gas on meter",
                     data_frame.time,
                     data_frame.data.electricity_delivering,
                     data_frame.data.electricity_delivered_t1,
                     data_frame.data.electricity_delivered_t2,
                     data_frame.data.gas_delivered,
            );

            // DSMR only does frames every 1 second.
            std::thread::sleep(Duration::from_millis(250));
        }
    }
}
