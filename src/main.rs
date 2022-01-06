use std::time::Duration;
use crate::port::PortBuilder;
use crate::reader::FrameReader;
use crate::data_frame::DataFrame;
use crate::parser::FrameParser;
use clap::Parser;

use crate::backend::Backend;
#[cfg(feature = "database")]
use crate::backend::Database;
#[cfg(feature = "api")]
use crate::backend::DSMRAPI;

mod port;
mod parser;
mod data_frame;
mod reader;
mod backend;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Path to the device or file for reading frames
    #[clap(short, long)]
    input: String,

    /// Database URL to write to
    #[cfg(feature = "database")]
    #[clap(long)]
    database: Option<String>,

    /// URL of the API server
    #[cfg(feature = "api")]
    #[clap(long="api")]
    api_url: Option<String>,

    /// Authentication key for the API server
    #[cfg(feature = "api")]
    #[clap(long)]
    api_key: Option<String>,

    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
}

fn main() {
    let args: Args = Args::parse();

    if args.api_url.is_some() && args.api_key.is_none() {
        println!("Option 'api-key' is required when using the api.");
        return;
    }

    let port = PortBuilder::from_path(&args.input);
    let mut frame_reader = FrameReader::new(port);

    // let mut backend = Database::new("postgres://pi:pi@localhost".to_string());
    let mut backend = make_backend(&args);
    backend.init().unwrap();

    loop {
        if let Some(raw_frame) = frame_reader.read_next_byte() {
            let data_frame = FrameParser::parse(raw_frame).unwrap();

            if args.verbose {
                println!("[{:?}]: {:?} kW ({:?} + {:?} kWh on meter), {:?} m3 gas on meter",
                         data_frame.time,
                         data_frame.data.electricity_delivering,
                         data_frame.data.electricity_delivered_t1,
                         data_frame.data.electricity_delivered_t2,
                         data_frame.data.gas_delivered,
                );
            }

            backend.send(&data_frame).unwrap();

            // DSMR only does frames every 1 second.
            std::thread::sleep(Duration::from_millis(250));
        }
    }
}

fn make_backend(args: &Args) -> Box<dyn Backend> {
    #[cfg(feature = "database")]
    if let Some(db_url) = &args.database {
        return Box::new(Database::new(db_url.as_str()));
    }

    #[cfg(feature = "api")]
    if let Some(api_url) = &args.api_url {
        if let Some(api_key) = &args.api_key {
            return Box::new(DSMRAPI::new(api_url.as_str(), api_key.as_str()));
        }
    }

    panic!("Either 'api' or 'database' is required'");
}
