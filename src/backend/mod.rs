use std::fmt::Error;
use crate::DataFrame;

#[cfg(feature = "api")]
mod api;

#[cfg(feature = "database")]
mod database;

#[cfg(feature = "api")]
pub use api::*;

#[cfg(feature = "database")]
pub use database::*;

pub trait Backend {
    fn init(&mut self) -> Result<(), Error>;
    fn send(&mut self, data_frame: &DataFrame) -> Result<(), Error>;
}
