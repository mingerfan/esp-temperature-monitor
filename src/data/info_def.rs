use core::fmt;

#[repr(C, align(4))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfoSlot {
    temperature: i16,
    humidity: u16,
}

impl fmt::Display for InfoSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InfoSlot {{ temperature: {:.1}°C, humidity: {:.1}% }}",
            self.get_temperature(),
            self.get_humidity()
        )
    }
}


impl InfoSlot {
    // pub const SERIALIZED_SIZE: usize = std::mem::size_of::<Self>();

    // pub fn new(temperature_tenths: i8, humidity_tenths: u8) -> Self {
    //     Self {
    //         temperature: temperature_tenths,
    //         humidity: humidity_tenths,
    //     }
    // }

    pub fn new_from_f32(temperature: f32, humidity: f32) -> Self {
        log::info!(
            "Creating InfoSlot from f32: temperature = {temperature:.1}, humidity = {humidity:.1}"
        );
        Self {
            temperature: (temperature * 10.0) as i16,
            humidity: (humidity * 10.0) as u16,
        }
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature as f32 / 10.0
    }

    pub fn get_humidity(&self) -> f32 {
        self.humidity as f32 / 10.0
    }

    // pub fn temperature_raw(&self) -> i8 {
    //     self.temperature
    // }

    // pub fn humidity_raw(&self) -> u8 {
    //     self.humidity
    // }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const Self) }
    }

    // pub fn set_temperature(&mut self, temperature: f32) {
    //     self.temperature = (temperature * 10.0) as i8;
    // }

    // pub fn set_humidity(&mut self, humidity: f32) {
    //     self.humidity = (humidity * 10.0) as u8;
    // }

}
