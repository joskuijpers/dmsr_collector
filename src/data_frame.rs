use chrono::Local;

#[derive(Debug)]
pub struct RawFrame {
    data: String,
}

impl RawFrame {
    pub(crate) fn new(data: String) -> Self {
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_data(&self) -> &str {
        self.data.as_str()
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Object {
    Version(u32), // ?, 1-3:0.2.8.255
    Time(chrono::DateTime<Local>), // 0-0:1.0.0.255
    Unknown(String, String),
}

#[derive(Debug)]
pub struct DataFrame {
    prefix: String,
    identifier: String,
    objects: Vec<Object>,
    checksum: u16,
}

impl DataFrame {
    pub fn new(prefix: String, identifier: String, objects: Vec<Object>, checksum: u16) -> Self {
        Self {
            prefix,
            identifier,
            objects,
            checksum,
        }
    }

    pub fn is_valid(&self) -> bool {
        true
    }
}
