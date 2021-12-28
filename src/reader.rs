use nom::AsBytes;
use crate::data_frame::RawFrame;
use crate::port::Port;

enum ReaderState {
    LookingForHeader,
    LookingForFooter,
    LookingForEnd,
}

pub struct FrameReader {
    state: ReaderState,
    buffer: Vec<u8>,
    port: Box<dyn Port>,
}

impl FrameReader {
    pub fn new(port: Box<dyn Port>) -> Self {
        Self {
            state: ReaderState::LookingForHeader,
            buffer: Default::default(),
            port,
        }
    }

    /// Read the next byte from the port, and once a whole frame is available,
    /// return it.
    pub fn read_next_byte(&mut self) -> Option<RawFrame> {
        self.port.fetch();

        if let Some(c) = self.port.read() {
            match self.state {
                ReaderState::LookingForHeader => {
                    if c == b'/' {
                        self.state = ReaderState::LookingForFooter;
                        self.buffer.push(c);
                    }
                }
                ReaderState::LookingForFooter => {
                    self.buffer.push(c);

                    if c == b'!' {
                        self.state = ReaderState::LookingForEnd
                    }
                }
                ReaderState::LookingForEnd => {
                    self.buffer.push(c);

                    if c == b'\n' {
                        let frame_data = match std::str::from_utf8(self.buffer.as_bytes()) {
                            Ok(frame_data) => Some(frame_data.to_string()),
                            Err(_) => None,
                        };

                        self.state = ReaderState::LookingForHeader;
                        self.buffer = Default::default();

                        return if let Some(frame_content) = frame_data {
                            Some(RawFrame::new(frame_content))
                        } else {
                            // Skip over invalid frame
                            None
                        }
                    }
                }
            }
        }

        None
    }

    /// Read next frame. Blocking when no data is available.
    fn read_next_frame(&mut self) -> RawFrame {
        loop {
            if let Some(raw_frame) = self.read_next_byte() {
                return raw_frame;
            }
        }
    }
}

impl Iterator for FrameReader {
    type Item = RawFrame;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.read_next_frame())
    }
}

#[cfg(test)]
mod tests {
    use crate::{FrameReader, PortBuilder};

    fn reader_from_str(str: &'static str) -> FrameReader {
        FrameReader::new(PortBuilder::from_data(str.as_bytes()))
    }

    #[test]
    fn port_result() {
        let reader = reader_from_str("/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0:1.0.0(211227133446W)\r\n!38AF\r\n/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0:1.0.0(211227133446W)\r\n!38AF\r\n");

        // We need to take only 2 because it blocks waiting on next frame
        let num_frames = reader.into_iter().take(2).count();
        assert_eq!(num_frames, 2);
    }

    #[test]
    fn single_frame_length() {
        let mut reader = reader_from_str("/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0:1.0.0(211227133446W)\r\n!38AF\r\n"); // 69

        let frame = reader.read_next_frame();
        assert_eq!(frame.len(), 69);
    }

    #[test]
    fn start_on_broken_frame() {
        let mut reader = reader_from_str("brokenframe/ISK5\\2M550E-1012\r\n\r\n1-3:0.2.8(50)\r\n0-0:1.0.0(211227133446W)\r\n!38AF\r\n"); // 69

        let frame = reader.read_next_frame();
        assert_eq!(frame.len(), 69);
    }
}
