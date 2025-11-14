use crate::data::info_def::InfoSlot;
use embedded_dht_rs::dht22::Dht22;
use esp_idf_svc::hal::{delay::Ets, gpio::PinDriver};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemperatureSensorError {
    #[error("传感器读取失败: {0}")]
    Read(String),
}

/// 温度传感器封装，目前支持 DHT22
pub struct TemperatureSensor {
    dht22: Dht22<PinDriver<'static, esp_idf_svc::hal::gpio::AnyIOPin, esp_idf_svc::hal::gpio::InputOutput>, Ets>,
}

impl TemperatureSensor {
    /// 创建新的温度传感器实例
    /// 
    /// # Arguments
    /// * `pin` - DHT22 数据引脚
    /// 
    /// # Returns
    /// * `Result<Self, TemperatureSensorError>` - 成功返回传感器实例，失败返回错误
    pub fn new(
        pin: PinDriver<'static, esp_idf_svc::hal::gpio::AnyIOPin, esp_idf_svc::hal::gpio::InputOutput>,
    ) -> Result<Self, TemperatureSensorError> {
        let dht22 = Dht22::new(pin, Ets);
        
        Ok(Self { dht22 })
    }

    /// 读取传感器数据并返回 InfoSlot
    /// 
    /// # Returns
    /// * `Result<InfoSlot, TemperatureSensorError>` - 成功返回温湿度数据，失败返回错误
    pub fn read_data(&mut self) -> Result<InfoSlot, TemperatureSensorError> {
        match self.dht22.read() {
            Ok(reading) => {
                let info_slot = InfoSlot::new_from_f32(reading.temperature, reading.humidity);
                log::debug!(
                    "传感器读取成功: 温度 {:.1}°C, 湿度 {:.1}%",
                    reading.temperature,
                    reading.humidity
                );
                Ok(info_slot)
            }
            Err(e) => {
                let error_msg = format!("DHT22 读取失败: {e:?}");
                log::error!("{error_msg}");
                Err(TemperatureSensorError::Read(error_msg))
            }
        }
    }

    // /// 尝试读取传感器数据，失败时返回 None 而不是错误
    // /// 适用于不希望因传感器读取失败而中断程序的场景
    // /// 
    // /// # Returns
    // /// * `Option<InfoSlot>` - 成功返回数据，失败返回 None
    // pub fn try_read_data(&mut self) -> Option<InfoSlot> {
    //     match self.read_data() {
    //         Ok(data) => Some(data),
    //         Err(e) => {
    //             log::warn!("传感器读取失败，返回 None: {e}");
    //             None
    //         }
    //     }
    // }

    // /// 获取原始的 DHT22 读取结果
    // /// 
    // /// # Returns
    // /// * `Result<SensorReading<f32>, TemperatureSensorError>` - 原始传感器数据
    // pub fn read_raw(&mut self) -> Result<SensorReading<f32>, TemperatureSensorError> {
    //     self.dht22.read().map_err(|e| {
    //         TemperatureSensorError::Read(format!("DHT22 原始读取失败: {e:?}"))
    //     })
    // }
}



