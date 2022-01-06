use std::fmt::Error;
use crate::DataFrame;

mod api;

#[cfg(feature = "database")]
mod database;

pub use api::*;

#[cfg(feature = "database")]
pub use database::*;

pub trait Backend {
    fn init(&mut self) -> Result<(), Error>;
    fn send(&mut self, data_frame: &DataFrame) -> Result<(), Error>;
}
