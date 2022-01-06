use std::fmt::Error;
use postgres::{Client, NoTls};
use crate::backend::Backend;
use crate::DataFrame;

pub struct Database {
    client: Client,
}

impl Database {
    pub fn new(url: &str) -> Self {
        let mut client = Client::connect(url, NoTls).unwrap();

        Self {
            client,
        }
    }
}

impl Backend for Database {
    fn init(&mut self) -> Result<(), Error> {
        self.client.batch_execute("
            CREATE TABLE IF NOT EXISTS dsmr_raw (
                id                  SERIAL PRIMARY KEY,
                time                TIMESTAMPTZ NOT NULL,
                delivering          DOUBLE PRECISION NOT NULL,
                delivered_t1        DOUBLE PRECISION NOT NULL,
                delivered_t2        DOUBLE PRECISION NOT NULL,
                gas_delivered       DOUBLE PRECISION NOT NULL
            )
        ").unwrap();

        Ok(())
    }

    fn send(&mut self, data_frame: &DataFrame) -> Result<(), Error> {
        self.client.execute(
            "INSERT INTO dsmr_raw (time, delivering, delivered_t1, delivered_t2, gas_delivered) VALUES ($1, $2, $3, $4, $5)",
            &[
                &data_frame.time,
                &data_frame.data.electricity_delivering,
                &data_frame.data.electricity_delivered_t1,
                &data_frame.data.electricity_delivered_t2,
                &data_frame.data.gas_delivered
            ],
        ).unwrap();

        Ok(())
    }
}
