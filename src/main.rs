mod config;
mod data;
mod macros;
mod peripherals;
mod service;
mod utils;

use service::ntp;
use std::thread::sleep;
use std::time::Duration;

use crate::data::info_def::InfoSlot;
use crate::peripherals::screen::{self, Screen, ScreenBuilder};
use crate::peripherals::temperature_sensor::TemperatureSensor;
use crate::peripherals::wifi::WifiBuilder;
use crate::utils::circular_queue;
// use embedded_hal::digital::{InputPin, OutputPin, PinState};

include!("../.env/config.rs");

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // 使用配置系统获取外设
    let (peripherals, gpio_config) = configure_peripherals!();

    // let mut random_generator = utils::rand::RandomGenerator::new();
    let mut time_db = data::time_db::TimeDB::new("temperature_db", 4096 * 5, true)?;

    // wifi 连接
    let wifi_buider = WifiBuilder::new(WIFI_SSID, WIFI_PASSWORD);
    let sysloop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;

    let wifi = wifi_buider.build(peripherals.modem, sysloop)?;
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
            .timeout(30) // 增加超时时间到 30 秒
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

    let mut temperature_sensor = TemperatureSensor::from_pin(gpio_config.temperature_pin)?;

    // 使用 ScreenBuilder 创建屏幕实例
    let mut screen = ScreenBuilder::with_pins(
        peripherals.spi2,
        gpio_config.spi_sck,  // SCK
        gpio_config.spi_mosi, // MOSI
        gpio_config.spi_cs,   // CS
        gpio_config.spi_dc,   // DC
    )?;

    let mut cnt = 10;
    loop {
        log::info!("主循环: 读取传感器数据并打印");
        // let info_slot = random_generator.get_info_slot();

        let info_slot = match temperature_sensor.read_data() {
            Ok(slot) => slot,
            Err(e) => {
                log::error!("读取传感器数据失败: {e}");
                sleep(Duration::from_secs(5));
                continue;
            }
        };

        // 使用 utils::time 获取 unix 时间戳
        let time = match utils::time::get_unix_timestamp() {
            Some(t) => t,
            None => {
                log::error!("获取当前时间失败");
                continue;
            }
        };
        // 使用 utils::time 格式化本地时间（东八区为 8*3600）
        let datetime_str = utils::time::get_formatted_time(
            "[year]-[month]-[day] [hour]:[minute]:[second]",
            8 * 3600,
        )
        .unwrap_or_else(|| "<时间格式化失败>".to_string());

        // 绘制时间
        screen.clear()?;
        let day_pos = screen::to_point(1, 7);
        screen.draw_text(&datetime_str[2..], day_pos)?;

        println!("读取到传感器数据({datetime_str}): {info_slot}");
        if time_db.insert(time, &info_slot).is_ok() {
            log::info!("已将数据存入数据库");
        } else {
            log::error!("将数据存入数据库失败");
        }

        // 使用英文绘制温度与湿度
        let temp_hum_str = format!(
            "TEMP:{:.1}°C\nHUMD:{:.1} %",
            info_slot.get_temperature(),
            info_slot.get_humidity()
        );
        let temp_hum_pos = screen::to_point(15, 30);
        screen.draw_text_big(&temp_hum_str, temp_hum_pos)?;

        screen.flush()?;

        sleep(Duration::from_secs(5));

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

    // screen.draw_example()?;

    loop {
        sleep(Duration::from_secs(1));
    }

    // Ok(())
}
