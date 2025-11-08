mod data;
mod peripherals;
mod utils;
mod service;
use peripherals::temperature_sensor::GetInfoSlot;
use std::thread::sleep;
use std::time::Duration;
use time::{format_description, OffsetDateTime};
use service::ntp;

use crate::peripherals::wifi::WifiBuilder;

include!("../.env/config.rs");

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = if let Ok(p) = esp_idf_svc::hal::peripherals::Peripherals::take() {
        p
    } else {
        log::error!("获取外设失败");
        return;
    };

    let mut random_generator = utils::rand::RandomGenerator::new();
    let mut time_db = match data::time_db::TimeDB::new("temperature_db", 4096, true) {
        Ok(db) => db,
        Err(e) => {
            log::error!("创建时间序列数据库失败: {e:?}");
            return;
        }
    };

    // wifi 连接
    let wifi_buider = WifiBuilder::new(WIFI_SSID, WIFI_PASSWORD);
    let sysloop = if let Ok(x) = esp_idf_svc::eventloop::EspSystemEventLoop::take() {
        x
    } else {
        log::error!("获取系统事件循环失败");
        return;
    };

    let modem = peripherals.modem;

    let wifi = match wifi_buider.build(modem, sysloop) {
        Ok(wifi) => wifi,
        Err(e) => {
            log::error!("WiFi 连接失败: {e:?}");
            return;
        }
    };
    log::info!("WiFi 已连接, IP 地址: {:?}", wifi.get_configuration());
    
    // 等待网络完全就绪
    log::info!("等待网络稳定...");
    sleep(Duration::from_secs(2));
    
    // 测试网络连接
    if !ntp::test_network_connectivity() {
        log::error!("网络连接不可用，跳过 NTP 同步");
        // 继续运行，但不同步时间
    } else {
        // 尝试同步时间
        log::info!("开始 NTP 时间同步...");
        let ntp_res = ntp::NtpConfig::new()
            .china_servers()
            .timeout(30)  // 增加超时时间到 30 秒
            .wait_for_sync(true)
            .init();

        match ntp_res {
            Ok(_sntp) => {
                log::info!("✅ NTP 时间同步成功");
            }
            Err(e) => {
                log::warn!("⚠️  NTP 时间同步失败: {e:?}，程序将继续运行");
                log::info!("💡 提示：可以尝试使用全局 NTP 服务器");
            }
        }
    }

    let mut cnt = 15;
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
        if time_db.insert(time + 350, &info_slot).is_ok() {
            log::info!("已将数据存入数据库");
        } else {
            log::error!("将数据存入数据库失败");
        }
        sleep(Duration::from_secs(1));

        // 数据读取
        if let Some(latest_slot) = time_db.latest() {
            log::info!("最新数据: {latest_slot}");
        } else {
            log::info!("数据库中无数据");
        }
        cnt -= 1;
        if cnt == 0 {
            break;
        }
    }

    // 打印所有温度数据
    let all_data = time_db.get_all_data();
    println!("数据库中所有温度数据，共 {} 条:", all_data.len());
    for slot in all_data {
        println!("{slot}");
    }
}
