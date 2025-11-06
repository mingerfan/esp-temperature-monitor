mod info;
mod peripherals;
mod utils;
use esp_idf_sys::{CONFIG_WL_SECTOR_SIZE, WL_INVALID_HANDLE, esp, wl_handle_t};
use peripherals::temperature_sensor::GetInfoSlot;
use std::thread::sleep;
use std::time::Duration;
use time::{format_description, OffsetDateTime};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // let peripherals = Peripherals::take().unwrap();

    // spiffs
    // let spiffs_cfg = esp_idf_sys::esp_vfs_spiffs_conf_t {
    //     base_path: c"/spiffs".as_ptr(), // C 字符串字面量（Rust 1.77+）
    //     partition_label: core::ptr::null(),
    //     max_files: 5,
    //     format_if_mount_failed: true,
    // };

    // let res = unsafe { esp_idf_sys::esp_vfs_spiffs_register(&spiffs_cfg) };
    // if esp!(res).is_err() {
    //     log::error!("Failed to mount SPIFFS");
    //     return;
    // }

    // fatfs
    let fatfs_cfg = esp_idf_sys::esp_vfs_fat_mount_config_t {
        max_files: 5,
        format_if_mount_failed: true,
        allocation_unit_size: CONFIG_WL_SECTOR_SIZE as usize,
        disk_status_check_enable: false,
    };

    let mut fat_handle: wl_handle_t = WL_INVALID_HANDLE;

    let res = unsafe {
        esp_idf_sys::esp_vfs_fat_spiflash_mount(
            c"/fatfs".as_ptr(),
            c"fatfs".as_ptr(),
            &fatfs_cfg,
            (&mut fat_handle) as *mut wl_handle_t,
        )
    };
    if esp!(res).is_err() {
        log::error!("Failed to mount FATFS");
        return;
    }
    // 先执行一次写入测试
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        let test_file_path = "/fatfs/test";
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .open(test_file_path)
        {
            Ok(f) => f,
            Err(e) => {
                log::error!("创建测试文件失败: {e:?}");
                return;
            }
        };
        if let Err(e) = file.write_all(b"FATFS test\n") {
            log::error!("写入测试文件失败: {e:?}");
            return;
        }
        log::info!("FATFS 写入测试成功");
        // 读取试试
        drop(file); // 关闭文件
        let content = match std::fs::read_to_string(test_file_path) {
            Ok(c) => c,
            Err(e) => {
                log::error!("读取测试文件失败: {e:?}");
                return;
            }
        };
        log::info!("读取测试文件内容: {content}");
        // 删除测试文件
        if let Err(e) = std::fs::remove_file(test_file_path) {
            log::error!("删除测试文件失败: {e:?}");
            return;
        }
    }

    let mut random_generator = utils::rand::RandomGenerator::new();
    let mut time_db = match info::time_db::TimeDB::new("/fatfs/temperature_db", 1024) {
        Ok(db) => db,
        Err(e) => {
            log::error!("创建时间序列数据库失败: {e:?}");
            return;
        }
    };

    loop {
        log::info!("主循环: 读取传感器数据并打印");
        let info_slot = random_generator.get_info_slot();
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let datetime_utc = OffsetDateTime::from_unix_timestamp(time);

        if datetime_utc.is_err() {
            log::error!("获取当前时间失败");
            continue;
        }
        let datetime_utc = datetime_utc.unwrap();
        let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
        let datetime_str = datetime_utc.format(&format).unwrap();

        println!("读取到传感器数据({datetime_str}): {info_slot}");
        if time_db.insert(time + 100000, &info_slot).is_ok() {
            log::info!("已将数据存入数据库");
        } else {
            log::error!("将数据存入数据库失败");
        }
        sleep(Duration::from_secs(10));

        // 数据读取
        // if let Some(latest_slot) = time_db.get_by_time(time, time).first() {
        //     log::info!("最新数据: {latest_slot}");
        // } else {
        //     log::info!("数据库中无数据");
        // }
    }
}
