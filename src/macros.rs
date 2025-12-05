//! 配置宏
//! 
//! 提供简化外设配置使用的宏

/// 配置外设的宏
/// 
/// 这个宏简化了从 GPIO 管理器获取配置的过程。
/// 返回一个元组 `(peripherals, gpio_config)`，其中：
/// - `peripherals`: 完整的 `Peripherals` 对象，用于访问 modem、SPI2 等其他外设
/// - `gpio_config`: `GPIOConfig` 对象，包含所有已配置的 GPIO 引脚
/// 
/// # 示例
/// ```
/// let (peripherals, gpio_config) = configure_peripherals!();
/// let temperature_sensor = TemperatureSensor::from_pin(gpio_config.temperature_pin)?;
/// ```
#[macro_export]
macro_rules! configure_peripherals {
    () => {{
        use $crate::config::{GPIOManager, pins::PIN_CONFIG};
        
        let manager = match GPIOManager::new() {
            Ok(manager) => manager,
            Err(e) => {
                log::error!("GPIO 管理器初始化失败: {}", e);
                return Err(anyhow::anyhow!("GPIO 管理器初始化失败: {}", e));
            }
        };
        
        match manager.configure(&PIN_CONFIG) {
            Ok((peripherals, gpio_config)) => (peripherals, gpio_config),
            Err(e) => {
                log::error!("GPIO 配置失败: {}", e);
                return Err(anyhow::anyhow!("GPIO 配置失败: {}", e));
            }
        }
    }};
    
    ($config:expr) => {{
        use $crate::config::GPIOManager;
        
        let manager = match GPIOManager::new() {
            Ok(manager) => manager,
            Err(e) => {
                log::error!("GPIO 管理器初始化失败: {}", e);
                return Err(anyhow::anyhow!("GPIO 管理器初始化失败: {}", e));
            }
        };
        
        match manager.configure($config) {
            Ok((peripherals, gpio_config)) => (peripherals, gpio_config),
            Err(e) => {
                log::error!("GPIO 配置失败: {}", e);
                return Err(anyhow::anyhow!("GPIO 配置失败: {}", e));
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
        use $crate::config::GPIOManager;
        
        let mut manager = match GPIOManager::new() {
            Ok(manager) => manager,
            Err(e) => {
                log::error!("GPIO 管理器初始化失败: {}", e);
                return Err(anyhow::anyhow!("GPIO 管理器初始化失败: {}", e));
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
