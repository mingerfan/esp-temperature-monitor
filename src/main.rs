mod info;
mod utils;
use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();


    // let shtcx_instance = shtcx::shtc3()

    loop {
        log::info!("Hello, world!");
        FreeRtos::delay_ms(500);
        FreeRtos::delay_ms(500);
    }
}
