//! 外设配置模块
//! 
//! 提供引脚和外设的配置定义和管理器

pub mod pins;
pub mod gpio_manager;

// 重新导出常用类型
pub use gpio_manager::GPIOManager;
pub use pins::PinConfig;

/// 默认引脚配置
/// 
/// 使用项目当前的引脚分配：
/// - 温度传感器: GPIO5
/// - SPI SCK: GPIO2
/// - SPI MOSI: GPIO0
/// - SPI CS: GPIO18
/// - SPI DC: GPIO12
pub const PIN_CONFIG: PinConfig = PinConfig {
    temperature_sensor: 5,
    spi_sck: 2,
    spi_mosi: 0,
    spi_cs: 18,
    spi_dc: 12,
};
