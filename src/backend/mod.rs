use std::fmt::Error;
use crate::DataFrame;

mod api;
mod database;

pub use api::*;
pub use database::*;

pub trait Backend {
    fn init(&mut self) -> Result<(), Error>;
    fn send(&mut self, data_frame: &DataFrame) -> Result<(), Error>;
}
