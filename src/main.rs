use std::time::Duration;
use postgres::{Client, NoTls};
use postgres::types::ToSql;
use crate::parser::Parser;
use crate::port::PortBuilder;
use crate::reader::FrameReader;

mod port;
mod parser;
mod data_frame;
mod reader;

fn main() {
    // Open port
    let port = PortBuilder::from_device("/dev/ttyUSB0");

    // Fake port
    // let port = PortBuilder::from_data(include_str!("../../tests/testdata").as_bytes());
    let mut frame_reader = FrameReader::new(port);

    let mut client = Client::connect("host=localhost user=pi password=pi", NoTls).unwrap();
    // let mut client = Client::connect("host=localhost user=postgres", NoTls).unwrap();

    client.batch_execute("
        CREATE TABLE IF NOT EXISTS dsmr_raw (
            id                  SERIAL PRIMARY KEY,
            time                TIMESTAMPTZ NOT NULL,
            delivering          DOUBLE PRECISION NOT NULL,
            delivered_t1        DOUBLE PRECISION NOT NULL,
            delivered_t2        DOUBLE PRECISION NOT NULL,
            gas_delivered       DOUBLE PRECISION NOT NULL
        )
    ").unwrap();

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

            client.execute(
                "INSERT INTO dsmr_raw (time, delivering, delivered_t1, delivered_t2, gas_delivered) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &data_frame.time,
                    &data_frame.data.electricity_delivering,
                    &data_frame.data.electricity_delivered_t1,
                    &data_frame.data.electricity_delivered_t2,
                    &data_frame.data.gas_delivered
                ],
            ).unwrap();

            // DSMR only does frames every 1 second.
            std::thread::sleep(Duration::from_millis(250));
        }
    }
}
