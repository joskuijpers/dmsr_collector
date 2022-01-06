use std::fmt::Error;
use crate::backend::Backend;
use crate::DataFrame;

pub struct DSMRAPI {
    // url
    // api key
    // queue
}

impl DSMRAPI {
    pub fn new(url: &str, key: &str) -> Self {
        Self {}
    }

    fn send_queue(&mut self) {
        // make JSON
        // send
        // clear queue
    }
}

impl Backend for DSMRAPI {
    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn send(&mut self, data_frame: &DataFrame) -> Result<(), Error> {
        println!("WRITE TO API");

        // add to a queue

        // if queue hits N, then send in bulk and clear queue
        self.send_queue();

        Ok(())
    }
}
