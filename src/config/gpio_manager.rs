//! GPIO 引脚管理器
//! 
//! 安全地管理 GPIO 引脚的所有权，防止冲突使用

use esp_idf_svc::hal::{
    gpio::AnyIOPin,
    peripheral::Peripheral,
    peripherals::Peripherals,
};
use std::collections::HashSet;
use thiserror::Error;

use crate::config::pins::PinConfig;

/// GPIO 引脚配置
/// 
/// 包含所有已配置的 GPIO 引脚，所有权已从管理器转移
pub struct GPIOConfig {
    /// 温度传感器引脚
    pub temperature_pin: AnyIOPin,
    
    /// SPI 时钟引脚
    pub spi_sck: AnyIOPin,
    
    /// SPI 主出从入引脚
    pub spi_mosi: AnyIOPin,
    
    /// SPI 片选引脚
    pub spi_cs: AnyIOPin,
    
    /// 屏幕数据/命令选择引脚
    pub spi_dc: AnyIOPin,
}

/// GPIO 管理器错误类型
#[derive(Debug, Error)]
pub enum GPIOError {
    #[error("引脚 {0} 已被使用")]
    PinAlreadyUsed(u8),
    
    #[error("无效的引脚编号: {0}")]
    InvalidPin(u8),
    
    #[error("GPIO 初始化失败: {0}")]
    GPIOInit(String),
}

/// GPIO 引脚管理器
/// 
/// 安全地管理 GPIO 引脚的所有权，使用 `clone_unchecked()` 允许多次访问，
/// 同时跟踪已使用的引脚防止冲突。
pub struct GPIOManager {
    peripherals: Peripherals,
    used_pins: HashSet<u8>,
}

impl GPIOManager {
    /// 创建新的 GPIO 管理器
    /// 
    /// # 返回
    /// * `Ok(Self)` - 管理器创建成功
    /// * `Err(GPIOError)` - 初始化失败
    pub fn new() -> Result<Self, GPIOError> {
        let peripherals = Peripherals::take()
            .map_err(|e| GPIOError::GPIOInit(format!("获取外设失败: {e}")))?;
            
        Ok(Self {
            peripherals,
            used_pins: HashSet::new(),
        })
    }
    
    /// 根据引脚配置获取 GPIO 引脚和完整的外设对象
    /// 
    /// # 参数
    /// * `config` - 引脚配置
    /// 
    /// # 返回
    /// * `Ok((Peripherals, GPIOConfig))` - 成功获取外设和 GPIO 配置
    /// * `Err(GPIOError)` - 配置失败
    /// 
    /// 返回一个元组，包含：
    /// 1. 完整的 `Peripherals` 对象（用于访问 modem、SPI2 等其他外设）
    /// 2. `GPIOConfig` 对象（包含所有已配置的 GPIO 引脚）
    pub fn configure(mut self, config: &PinConfig) -> Result<(Peripherals, GPIOConfig), GPIOError> {
        // 验证配置
        crate::config::pins::validate_config(config)
            .map_err(GPIOError::GPIOInit)?;
        
        // 获取所有需要的引脚
        let temperature_pin = self.take_gpio(config.temperature_sensor)?;
        let spi_sck = self.take_gpio(config.spi_sck)?;
        let spi_mosi = self.take_gpio(config.spi_mosi)?;
        let spi_cs = self.take_gpio(config.spi_cs)?;
        let spi_dc = self.take_gpio(config.spi_dc)?;
        
        Ok((
            self.peripherals,
            GPIOConfig {
                temperature_pin,
                spi_sck,
                spi_mosi,
                spi_cs,
                spi_dc,
            }
        ))
    }
    
    /// 安全地获取 GPIO 引脚
    /// 
    /// 使用 `clone_unchecked()` 创建引脚的克隆，同时跟踪已使用的引脚。
    /// 
    /// # 参数
    /// * `pin_num` - GPIO 引脚编号
    /// 
    /// # 返回
    /// * `Ok(AnyIOPin)` - 引脚获取成功
    /// * `Err(GPIOError)` - 引脚已被使用或无效
    pub fn take_gpio(&mut self, pin_num: u8) -> Result<AnyIOPin, GPIOError> {
        // 检查引脚是否已被使用
        if self.used_pins.contains(&pin_num) {
            return Err(GPIOError::PinAlreadyUsed(pin_num));
        }
        
        // 获取引脚并转换为 AnyIOPin
        // 根据项目实际使用的引脚和常见的 ESP32 GPIO 引脚
        let pin = match pin_num {
            0 => unsafe { self.peripherals.pins.gpio0.clone_unchecked() }.into(),
            1 => unsafe { self.peripherals.pins.gpio1.clone_unchecked() }.into(),
            2 => unsafe { self.peripherals.pins.gpio2.clone_unchecked() }.into(),
            3 => unsafe { self.peripherals.pins.gpio3.clone_unchecked() }.into(),
            4 => unsafe { self.peripherals.pins.gpio4.clone_unchecked() }.into(),
            5 => unsafe { self.peripherals.pins.gpio5.clone_unchecked() }.into(),
            6 => unsafe { self.peripherals.pins.gpio6.clone_unchecked() }.into(),
            7 => unsafe { self.peripherals.pins.gpio7.clone_unchecked() }.into(),
            8 => unsafe { self.peripherals.pins.gpio8.clone_unchecked() }.into(),
            9 => unsafe { self.peripherals.pins.gpio9.clone_unchecked() }.into(),
            10 => unsafe { self.peripherals.pins.gpio10.clone_unchecked() }.into(),
            11 => unsafe { self.peripherals.pins.gpio11.clone_unchecked() }.into(),
            12 => unsafe { self.peripherals.pins.gpio12.clone_unchecked() }.into(),
            13 => unsafe { self.peripherals.pins.gpio13.clone_unchecked() }.into(),
            14 => unsafe { self.peripherals.pins.gpio14.clone_unchecked() }.into(),
            15 => unsafe { self.peripherals.pins.gpio15.clone_unchecked() }.into(),
            16 => unsafe { self.peripherals.pins.gpio16.clone_unchecked() }.into(),
            17 => unsafe { self.peripherals.pins.gpio17.clone_unchecked() }.into(),
            18 => unsafe { self.peripherals.pins.gpio18.clone_unchecked() }.into(),
            19 => unsafe { self.peripherals.pins.gpio19.clone_unchecked() }.into(),
            21 => unsafe { self.peripherals.pins.gpio21.clone_unchecked() }.into(),
            // 注意：某些 GPIO 引脚可能不可用，根据实际硬件调整
            _ => return Err(GPIOError::InvalidPin(pin_num)),
        };
        
        // 标记引脚为已使用
        self.used_pins.insert(pin_num);
        Ok(pin)
    }
}

