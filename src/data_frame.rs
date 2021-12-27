use chrono::{DateTime, Local};

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

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Version(u32), // ?, 1-3:0.2.8.255
    Time(chrono::DateTime<Local>), // 0-0:1.0.0.255

    ElectricityDeliveredT1(f64), // 1-0:1.8.1.255   F9(3,3), tag 6
    ElectricityDeliveredT2(f64), // 1-0:1.8.2.255

    ElectricityDelivered(f64), // 1-0:1.7.0.255   F5(3,3), tag 18
    ElectricityReceived(f64), // 1-0:2.7.0.255

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

    pub fn time(&self) -> Option<DateTime<Local>> {
        let item = self.objects.iter().find(|s| match s {
            Object::Time(_) => true,
            _ => false,
        });

        if let Some(Object::Time(t)) = item {
            Some(*t)
        } else {
            None
        }
    }

    /// Get the amount of electricity being delivered in kW.
    pub fn electricity_delivering(&self) -> Option<f64> {
        let item = self.objects.iter().find(|s| match s {
            Object::ElectricityDelivered(_) => true,
            _ => false,
        });

        if let Some(Object::ElectricityDelivered(v)) = item {
            Some(*v)
        } else {
            None
        }
    }
}
