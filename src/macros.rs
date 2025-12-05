//! 配置宏
//! 
//! 提供简化外设配置使用的宏

/// 配置外设的宏
/// 
/// 这个宏简化了从外设管理器获取配置的过程。
/// 
/// # 示例
/// ```
/// let peripherals = configure_peripherals!();
/// let temperature_sensor = TemperatureSensor::from_pin(peripherals.temperature_pin)?;
/// ```
#[macro_export]
macro_rules! configure_peripherals {
    () => {{
        use $crate::config::{PeripheralManager, pins::PIN_CONFIG};
        
        let manager = match PeripheralManager::new() {
            Ok(manager) => manager,
            Err(e) => {
                log::error!("外设管理器初始化失败: {}", e);
                return Err(anyhow::anyhow!("外设管理器初始化失败: {}", e));
            }
        };
        
        match manager.configure(&PIN_CONFIG) {
            Ok(peripherals) => peripherals,
            Err(e) => {
                log::error!("外设配置失败: {}", e);
                return Err(anyhow::anyhow!("外设配置失败: {}", e));
            }
        }
    }};
    
    ($config:expr) => {{
        use $crate::config::PeripheralManager;
        
        let manager = match PeripheralManager::new() {
            Ok(manager) => manager,
            Err(e) => {
                log::error!("外设管理器初始化失败: {}", e);
                return Err(anyhow::anyhow!("外设管理器初始化失败: {}", e));
            }
        };
        
        match manager.configure($config) {
            Ok(peripherals) => peripherals,
            Err(e) => {
                log::error!("外设配置失败: {}", e);
                return Err(anyhow::anyhow!("外设配置失败: {}", e));
            }
        }
    }};
}

/// 快速获取 GPIO 引脚的宏
/// 
/// # 示例
/// ```
/// let pin5 = get_gpio!(5);
/// ```
#[macro_export]
macro_rules! get_gpio {
    ($pin_num:expr) => {{
        use $crate::config::PeripheralManager;
        
        let mut manager = match PeripheralManager::new() {
            Ok(manager) => manager,
            Err(e) => {
                log::error!("外设管理器初始化失败: {}", e);
                return Err(anyhow::anyhow!("外设管理器初始化失败: {}", e));
            }
        };
        
        match manager.take_gpio($pin_num) {
            Ok(pin) => pin,
            Err(e) => {
                log::error!("获取 GPIO 引脚失败: {}", e);
                return Err(anyhow::anyhow!("获取 GPIO 引脚失败: {}", e));
            }
        }
    }};
}
