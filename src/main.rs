mod info;
mod utils;
use esp_idf_sys::esp;
use info::info_def::InfoSlot;
use info::info_storage::InfoStorage;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // let peripherals = Peripherals::take().unwrap();

    let spiffs_cfg = esp_idf_sys::esp_vfs_spiffs_conf_t {
        base_path: c"/spiffs".as_ptr(), // C 字符串字面量（Rust 1.77+）
        partition_label: core::ptr::null(),
        max_files: 5,
        format_if_mount_failed: true,
    };

    let res = unsafe { esp_idf_sys::esp_vfs_spiffs_register(&spiffs_cfg) };
    esp!(res).unwrap();
    let mut storage = match InfoStorage::new() {
        Ok(storage) => storage,
        Err(err) => {
            log::error!("初始化 InfoStorage 失败: {err}");
            return;
        }
    };

    log::info!(
        "持久化仓恢复完成，当前条目数 {}/{}",
        storage.len(),
        storage.capacity()
    );

    if let Ok(existing) = storage.load_all() {
        if !existing.is_empty() {
            let sample_count = existing.len().min(3);
            for slot in existing.iter().rev().take(sample_count) {
                log::info!("恢复条目: {}", format_slot(slot));
            }
        }
    }

    let mut counter: u32 = 0;

    loop {
        let timestamp_secs = unsafe { esp_idf_sys::esp_timer_get_time() } / 1_000_000;
        let timestamp = timestamp_secs as u32;
        let temp_tenths = (((counter % 80) as i16) - 20) as i8;
        let humidity_tenths = ((counter * 7) % 100) as u8;
        let slot = InfoSlot::new(timestamp, temp_tenths, humidity_tenths);

        match storage.enqueue(&slot) {
            Ok(()) => log::info!(
                "写入条目: {} (总量 {}/{})",
                format_slot(&slot),
                storage.len(),
                storage.capacity()
            ),
            Err(err) => log::error!("写入环形存储失败: {err}"),
        }

        if storage.len() > 30 {
            match storage.dequeue() {
                Ok(oldest) => log::info!("移除最旧条目: {}", format_slot(&oldest)),
                Err(err) => log::warn!("移除最旧条目失败: {err}"),
            }
        }

        let start_range = timestamp.saturating_sub(60);
        match storage.find_range(start_range, timestamp) {
            Ok(samples) => {
                if let Some(latest) = samples.last() {
                    log::info!(
                        "最近一分钟共有 {} 条记录，最新: {}",
                        samples.len(),
                        format_slot(latest)
                    );
                } else {
                    log::info!("最近一分钟没有匹配的记录");
                }
            }
            Err(err) => log::warn!("查询范围失败: {err}"),
        }

        counter = counter.wrapping_add(1);
        sleep(Duration::from_secs(1));
    }
}

fn format_slot(slot: &InfoSlot) -> String {
    format!(
        "时间 {} 温度 {:.1}℃ 湿度 {:.1}%",
        slot.get_unix_time(),
        slot.get_temperature(),
        slot.get_humidity()
    )
}
