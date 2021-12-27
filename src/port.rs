use std::fmt::{Debug, Display, Formatter};
use std::io::{BufReader, BufRead};
use std::time::Duration;
use serialport::{DataBits, Parity, SerialPort, StopBits};
use crate::data_frame::RawFrame;
// use std::time::Duration;
// use serialport::{DataBits, Parity, SerialPort, StopBits};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    // Port(dyn std::error::Error),
    NoData,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => Display::fmt(&e, f),
            Error::Utf8(e) => Display::fmt(&e, f),
            Error::NoData => f.write_str("No data"),
        }
    }
}

impl std::error::Error for Error {}

pub struct USBPort {
    serialport: Box<dyn SerialPort>,
    buffer: Vec<u8>,
    buffer_pos: usize,
}

impl USBPort {
    pub fn new(dev_str: &str) -> Self {
        let port = serialport::new(dev_str, 115_200)
            .timeout(Duration::from_millis(10))
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .data_bits(DataBits::Eight)
            // 1 start bit
            .open()
            .expect("Port does not exist"); // TODO error forwarding

        Self {
            serialport: port,
            buffer: vec![0; 2048],
            buffer_pos: 0,
        }
    }

    // TODO: do something useful, like finding port
    pub fn list() {
        let ports = serialport::available_ports().expect("No ports found!");
        for p in ports {
            println!("{}", p.port_name);
        }
    }
}

// impl Iterator for USBPort {
//     type Item = Result<RawFrame, Error>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         // TODO: buffer has to be a ring buffer..... or Vec and removing elements!
//         //  code below will not work
//
//         // Read data and add to buffer
//         let size = match self.serialport.read(self.buffer.as_mut_slice()) {
//             Ok(size) => size,
//             Err(e) => return Some(Err(Error::Port(e))),
//         };
//
//         self.buffer_pos += size;
//
//         // Try to parse a frame. It might not succeed and more data is needed.
//         let result = match try_parse(self.buffer.as_bytes()) {
//             Ok(result) => result,
//             Err(e) => return Some(Err(e)),
//         };
//
//         // Progress in buffer
//         self.buffer_pos -= result.1;
//
//         Some(Ok(raw_frame))
//     }
// }

/// Port using a file as input for testing.
/// Will cut into 100 byte blocks to test continuous stream.
///
/// Only works up to 8kb, so just testing
pub struct FilePort<'a> {
    reader: BufReader<&'a [u8]>,
}

impl<'a> FilePort<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let reader = BufReader::new(data);

        Self {
            reader,
        }
    }
}

fn try_parse(buffer: &[u8]) -> Result<(RawFrame, usize), Error> {
    // Find the ! and then skip until CRLF
    let frame_start = buffer
        .iter()
        .take_while(|c| **c != b'/')
        .count();

    // Count until start of the CRC footer
    let frame_before_crc = buffer
        .iter()
        .skip(frame_start)
        .take_while(|c| **c != b'!')
        .count() + frame_start;

    // Then skip the footer
    let crc_size = buffer
        .iter()
        .skip(frame_before_crc)
        .take_while(|c| **c != b'\n')
        .count() + 1;

    let frame_end = frame_before_crc + crc_size;

    if frame_end > buffer.len() {
        return Err(Error::NoData);
    }

    // println!("Frame starts from {} and size {}", frame_start, frame_end - frame_start);
    // println!("Frame before CRC {}, CRC size {}", frame_before_crc, crc_size);

    // Combine line from data until the newlines
    let frame_data = match std::str::from_utf8(&buffer[frame_start..frame_end]) {
        Ok(frame_data) => frame_data.to_string(),
        Err(e) => return Err(Error::Utf8(e)),
    };

    Ok((RawFrame::new(frame_data), frame_end))
}

impl<'a> Iterator for FilePort<'a> {
    type Item = Result<RawFrame, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let (raw_frame, total) = {
            let buffer = match self.reader.fill_buf() {
                Ok(buffer) => buffer,
                Err(e) => return Some(Err(Error::Io(e))),
            };

            // No items, allow waiting for some.
            if buffer.is_empty() {
                return Some(Err(Error::NoData));
            }

            let result = match try_parse(buffer) {
                Ok(result) => result,
                Err(e) => return Some(Err(e)),
            };

            result
        };
        self.reader.consume(total);

        Some(Ok(raw_frame))
    }
}

#[cfg(test)]
mod tests {
    use crate::FilePort;
    use crate::port::try_parse;

    #[test]
    fn port_result() {
        let input = "/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0:1.0.0(211227133446W)\r\n!38AF\r\n/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0:1.0.0(211227133446W)\r\n!38AF\r\n";
        
        let port = FilePort::new(input.as_bytes());

        let num_frames = port.into_iter().take(2).count();
        assert_eq!(num_frames, 2);
    }

    #[test]
    fn single_frame_length() {
        let input = "/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0:1.0.0(211227133446W)\r\n!38AF\r\n"; // 69

        let r = try_parse(input.as_bytes()).unwrap();
        assert_eq!(r.1, input.len());
        assert_eq!(r.1, r.0.len());
    }

    #[test]
    fn start_on_broken_frame() {
        let input = "brokenframe/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0:1.0.0(211227133446W)\r\n!38AF\r\n"; // 69

        let r = try_parse(input.as_bytes()).unwrap();

        // Jump to frame end
        assert_eq!(r.1, input.len());

        // Frame length must be right
        assert_eq!(r.0.len(), 69);
    }

    #[test]
    fn handle_partial() {
        let input = "brokenframe/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0";

        let r = try_parse(input.as_bytes());
        r.err().unwrap();
    }
}
