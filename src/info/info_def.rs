
#[repr(C, align(4))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfoSlot {
    timpstamp: u32,
    temperature: i8,
    humidity: u8
}


impl InfoSlot {
    pub fn get_temperature(&self) -> f32 {
        self.temperature as f32 / 10.0
    }

    pub fn get_humidity(&self) -> f32 {
        self.humidity as f32 / 10.0
    }

    pub fn get_unix_time(&self) -> u32 {
        self.timpstamp
    }

    // setter
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = (temperature * 10.0) as i8;
    }
    
    pub fn set_humidity(&mut self, humidity: f32) {
        self.humidity = (humidity * 10.0) as u8;
    }

    pub fn set_unix_time(&mut self, timpstamp: u32) {
        self.timpstamp = timpstamp;
    }
}