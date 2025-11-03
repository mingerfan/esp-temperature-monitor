#[repr(C, align(4))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfoSlot {
    timestamp: u32,
    temperature: i8,
    humidity: u8,
}

impl InfoSlot {
    pub const SERIALIZED_SIZE: usize = 6;

    pub fn new(timestamp: u32, temperature_tenths: i8, humidity_tenths: u8) -> Self {
        Self {
            timestamp,
            temperature: temperature_tenths,
            humidity: humidity_tenths,
        }
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature as f32 / 10.0
    }

    pub fn get_humidity(&self) -> f32 {
        self.humidity as f32 / 10.0
    }

    pub fn get_unix_time(&self) -> u32 {
        self.timestamp
    }

    pub fn temperature_raw(&self) -> i8 {
        self.temperature
    }

    pub fn humidity_raw(&self) -> u8 {
        self.humidity
    }

    pub fn timestamp_raw(&self) -> u32 {
        self.timestamp
    }

    pub fn as_bytes(&self) -> [u8; Self::SERIALIZED_SIZE] {
        let mut buf = [0u8; Self::SERIALIZED_SIZE];
        buf[..4].copy_from_slice(&self.timestamp.to_le_bytes());
        buf[4] = self.temperature as u8;
        buf[5] = self.humidity;
        buf
    }

    pub fn from_bytes(bytes: [u8; Self::SERIALIZED_SIZE]) -> Self {
        let mut timestamp_bytes = [0u8; 4];
        timestamp_bytes.copy_from_slice(&bytes[..4]);
        Self {
            timestamp: u32::from_le_bytes(timestamp_bytes),
            temperature: bytes[4] as i8,
            humidity: bytes[5],
        }
    }

    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = (temperature * 10.0) as i8;
    }

    pub fn set_humidity(&mut self, humidity: f32) {
        self.humidity = (humidity * 10.0) as u8;
    }

    pub fn set_unix_time(&mut self, timestamp: u32) {
        self.timestamp = timestamp;
    }
}
