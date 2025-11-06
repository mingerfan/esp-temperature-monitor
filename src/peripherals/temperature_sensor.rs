use crate::info::info_def;
use crate::utils::rand;


pub trait Temperature {
    fn read_temperature(&mut self) -> f32;
}

pub trait Humidity {
    fn read_humidity(&mut self) -> f32;
}

pub trait GetInfoSlot: Temperature + Humidity {
    fn get_info_slot(&mut self) -> info_def::InfoSlot {
        info_def::InfoSlot::new_from_f32(self.read_temperature(), self.read_humidity())
    }
}

impl <T: Temperature + Humidity> GetInfoSlot for T {}

impl Temperature for rand::RandomGenerator {
    fn read_temperature(&mut self) -> f32 {
        ((self.next_u32() % 100) as i32 - 30) as f32
    }
}

impl Humidity for rand::RandomGenerator {
    fn read_humidity(&mut self) -> f32 {
        (self.next_u32() % 100) as f32
    }
}

