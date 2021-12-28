use std::io::{BufReader, Write, Read};
use std::io::ErrorKind::TimedOut;
use std::time::Duration;
use ringbuf::{Consumer, Producer, RingBuffer};
use serialport::{DataBits, Parity, SerialPort, StopBits};

pub struct PortBuilder;
impl PortBuilder {
    pub fn from_data(data: &'static [u8]) -> Box<dyn Port> {
        Box::new(FilePort::new(data))
    }

    pub fn from_device(dev_str: &str) -> Box<dyn Port> {
        Box::new(USBPort::new(dev_str))
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
    fn new(dev_str: &str) -> Self {
        let port = serialport::new(dev_str, 115_200)
            .timeout(Duration::from_millis(10))
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

    // TODO: do something useful, like finding port
    // pub fn list() {
    //     let ports = serialport::available_ports().expect("No ports found!");
    //     for p in ports {
    //         println!("{}", p.port_name);
    //     }
    // }
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

        self.producer.write_all(&buffer[..size]).unwrap();
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
