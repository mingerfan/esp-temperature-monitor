mod info;
mod utils;
use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};
use info::info_storage::NvsInfoStorage;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // let peripherals = Peripherals::take().unwrap();

    let mut nvs_storage = NvsInfoStorage::new().unwrap();

    loop {
        log::info!("Hello, world!");
        FreeRtos::delay_ms(500);
        FreeRtos::delay_ms(500);
        let value = nvs_storage.nvs_read_u8("test_key").unwrap();
        log::info!("Read value from NVS: {value}");
    }
}
