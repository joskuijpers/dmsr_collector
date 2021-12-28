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
    Time(DateTime<Local>), // 0-0:1.0.0.255

    ElectricityDeliveredT1(f64), // 1-0:1.8.1.255   F9(3,3), tag 6
    ElectricityDeliveredT2(f64), // 1-0:1.8.2.255

    ElectricityDelivering(f64), // 1-0:1.7.0.255   F5(3,3), tag 18
    ElectricityReceiving(f64), // 1-0:2.7.0.255

    GasDelivered(DateTime<Local>, f64),

    Unknown(String, String),
}

#[derive(Debug)]
pub struct DataFrame {
    prefix: String,
    identifier: String,
    checksum: u16,

    pub version: u32,
    pub time: DateTime<Local>,
    pub data: DataFrameData,
}

#[derive(Debug, Default)]
pub struct DataFrameData {
    pub electricity_delivered_t1: f64,
    pub electricity_delivered_t2: f64,

    pub electricity_delivering: f64,
    pub electricity_receiving: f64,

    pub gas_delivered: f64,
}

impl DataFrame {
    pub fn new(prefix: String, identifier: String, objects: Vec<Object>, checksum: u16) -> Self {
        let mut data = DataFrameData::default();

        let mut version: u32 = 0;
        let mut time = Local::now();

        for object in objects.iter() {
            match object {
                Object::Version(v) => version = *v,
                Object::Time(t) => time = t.clone(),
                Object::ElectricityDeliveredT1(v) => data.electricity_delivered_t1 = *v,
                Object::ElectricityDeliveredT2(v) => data.electricity_delivered_t2 = *v,
                Object::ElectricityDelivering(v) => data.electricity_delivering = *v,
                Object::ElectricityReceiving(v) => data.electricity_receiving = *v,
                Object::GasDelivered(_, v) => data.gas_delivered = *v,
                Object::Unknown(_, _) => {}
            }
        }

        Self {
            prefix,
            identifier,
            data,
            checksum,
            time,
            version,
        }
    }

    pub fn is_valid(&self) -> bool {
        true // no validation
    }
}
