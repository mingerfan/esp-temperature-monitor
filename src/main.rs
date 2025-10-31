mod info;
mod utils;
use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_sys::esp;
use info::info_storage::NvsInfoStorage;
use esp_storage::{FlashStorage};
use embedded_storage::ReadStorage;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // let peripherals = Peripherals::take().unwrap();

    // let mut nvs_storage = NvsInfoStorage::new().unwrap();

    // let mut flash = FlashStorage::new();
    // let mut buf = [0u8; 256];
    // flash.read(0x9000, &mut buf).unwrap();

    let spiffs_cfg = esp_idf_sys::esp_vfs_spiffs_conf_t {
        base_path: c"/spiffs".as_ptr(),  // C 字符串字面量（Rust 1.77+）
        partition_label: core::ptr::null(),
        max_files: 5,
        format_if_mount_failed: true,
    };

    let res = unsafe { esp_idf_sys::esp_vfs_spiffs_register(&spiffs_cfg) };
    esp!(res).unwrap();

    // 先写入文件
    std::fs::write("/spiffs/hello.txt", b"Hello from SPIFFS!").unwrap();
    log::info!("File written to SPIFFS");

    // 再读取文件
    let content = std::fs::read_to_string("/spiffs/hello.txt").unwrap();
    log::info!("File content from SPIFFS: {}", content);

    // log::info!("Read data from Flash: {:x?}", &buf[..16]);

    loop {
        log::info!("Hello, world!");
        FreeRtos::delay_ms(500);
        FreeRtos::delay_ms(500);
        // let value = nvs_storage.nvs_read_u8("test_key").unwrap();
        // log::info!("Read value from NVS: {value}");
        
    }
}
