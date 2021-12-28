use std::io::{BufReader, Write, Read};
use std::io::ErrorKind::TimedOut;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use ringbuf::{Consumer, Producer, RingBuffer};
use serialport::{DataBits, Parity, SerialPort, StopBits};

pub struct PortBuilder;
impl PortBuilder {
    // pub fn from_path<P: AsRef<Path>>(path: P) -> Box<dyn Port> {
    //     let x = path.as_ref();
    //
    //     if let Some(device) = PortBuilder::get_serial_devices()
    //         .iter()
    //         .find(|p| *p == &x) {
    //         Self::from_device(device.to_str().unwrap())
    //     } else {
    //         Self::from_data(path)
    //     }
    // }

    pub fn from_data(data: &'static [u8]) -> Box<dyn Port> {
        Box::new(FilePort::new(data))
    }

    pub fn from_device<P: AsRef<Path>>(path: P) -> Box<dyn Port> {
        Box::new(USBPort::new(path))
    }

    /// Get a list of port paths
    fn get_serial_devices() -> Vec<PathBuf> {
        serialport::available_ports()
            .unwrap_or(Vec::new())
            .iter()
            .flat_map(|p|  PathBuf::from_str(&p.port_name))
            .collect()
    }
}

pub trait Port {
    /// Fetch values from the data source into intermediate buffers, if needed.
    fn fetch(&mut self);
    /// Read a single byte.
    fn read(&mut self) -> Option<u8>;
}

pub struct USBPort {
    serialport: Box<dyn SerialPort>,
    producer: Producer<u8>,
    consumer: Consumer<u8>,
}

impl USBPort {
    fn new<P: AsRef<Path>>(dev_path: P) -> Self {
        let os_path = dev_path.as_ref().to_str().unwrap();

        let port = serialport::new(os_path, 115_200)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .data_bits(DataBits::Eight)
            // 1 start bit
            .open()
            .expect("Port does not exist"); // TODO error forwarding

        let ringbuffer = RingBuffer::new(4096);
        let (producer, consumer) = ringbuffer.split();

        Self {
            serialport: port,
            producer,
            consumer,
        }
    }
}

impl Port for USBPort {
    fn fetch(&mut self) {
        let mut buffer = [0; 1024];

        // Read data and add to buffer
        let size = match self.serialport.read(buffer.as_mut()) {
            Ok(size) => size,
            Err(e) => {
                // Timeing out is regular behavior
                if e.kind() != TimedOut {
                    println!("ERROR: Failed to read from serial port {:?}", e);
                }
                return
            },
        };

        if size > 0 {
            self.producer.write(&buffer[..size]).unwrap();
        }
    }

    fn read(&mut self) -> Option<u8> {
        if self.consumer.is_empty() {
            return None
        } else {
            let mut buf: [u8; 1] = [0];

            self.consumer.read_exact(buf.as_mut()).unwrap();

            Some(buf[0])
        }
    }
}

/// Port with a byte array as input. Useful for testing without actual serial port.
pub struct FilePort {
    reader: BufReader<&'static [u8]>,
}

impl FilePort {
    fn new(data: &'static [u8]) -> Self {
        let reader = BufReader::new(data);

        Self {
            reader,
        }
    }
}

impl Port for FilePort {
    fn fetch(&mut self) {}

    fn read(&mut self) -> Option<u8> {
        let mut buf: [u8; 1] = [0];
        if let Ok(_) = self.reader.read_exact(buf.as_mut()) {
            Some(buf[0])
        } else {
            None
        }
    }
}
