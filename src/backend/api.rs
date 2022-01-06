use std::fmt::Error;
use crate::backend::Backend;
use crate::DataFrame;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct ElectricityFrame {
    t1: f64,
    t2: f64,
    delivering: f64,
    receiving: f64,
}

#[derive(Serialize, Debug)]
struct GasFrame {
    delivered: f64,
}

#[derive(Serialize, Debug)]
struct TransferFrame {
    time: String,

    electricity: ElectricityFrame,
    gas: GasFrame,
}

#[derive(Serialize, Debug)]
struct Transfer {
    frames: Vec<TransferFrame>,
}

pub struct DSMRAPI {
    queue: Vec<DataFrame>,
    url: String,
    authorization: String,
}

impl DSMRAPI {
    pub fn new(url: &str, key: &str) -> Self {
        Self {
            queue: Default::default(),
            url: format!("{}/api/v1/collect", url),
            authorization: format!("Bearer {}", key),
        }
    }

    fn send_queue(&mut self) {
        // drain queue, mapping into transfer frames
        let frames: Vec<TransferFrame> = self.queue
            .drain(..)
            .map(|df| TransferFrame {
                time: df.time.to_string(),
                electricity: ElectricityFrame {
                    t1: df.data.electricity_delivered_t1,
                    t2: df.data.electricity_delivered_t2,
                    delivering: df.data.electricity_delivering,
                    receiving: df.data.electricity_receiving
                },
                gas: GasFrame {
                    delivered: df.data.gas_delivered,
                },
            })
            .collect();

        let dto = Transfer { frames };

        let client = reqwest::blocking::Client::new();
        let res = client.post(&self.url)
            .header(reqwest::header::AUTHORIZATION, &self.authorization)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&dto)
            .send()
            .unwrap();
    }
}

impl Backend for DSMRAPI {
    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn send(&mut self, data_frame: &DataFrame) -> Result<(), Error> {
        self.queue.push(data_frame.clone());

        // Every 5 seconds
        if self.queue.len() > 5 {
            self.send_queue();
        }

        Ok(())
    }
}
