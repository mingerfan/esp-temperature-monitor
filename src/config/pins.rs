//! 引脚配置定义
//! 
//! 定义所有外设使用的 GPIO 引脚配置

/// 引脚配置结构体
/// 
/// 包含所有外设使用的 GPIO 引脚编号
#[derive(Debug, Clone, Copy)]
pub struct PinConfig {
    /// DHT22 温度传感器数据引脚 (GPIO5)
    pub temperature_sensor: u8,
    
    /// SPI 时钟引脚 (GPIO2)
    pub spi_sck: u8,
    
    /// SPI 主出从入引脚 (GPIO0)
    pub spi_mosi: u8,
    
    /// SPI 片选引脚 (GPIO18)
    pub spi_cs: u8,
    
    /// 屏幕数据/命令选择引脚 (GPIO12)
    pub spi_dc: u8,
}

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

/// 验证引脚配置的有效性
/// 
/// # 参数
/// * `config` - 要验证的引脚配置
/// 
/// # 返回
/// * `Ok(())` - 配置有效
/// * `Err(String)` - 配置无效，包含错误信息
pub fn validate_config(config: &PinConfig) -> Result<(), String> {
    let pins = [
        config.temperature_sensor,
        config.spi_sck,
        config.spi_mosi,
        config.spi_cs,
        config.spi_dc,
    ];
    
    // 检查是否有重复的引脚
    for i in 0..pins.len() {
        for j in (i + 1)..pins.len() {
            if pins[i] == pins[j] {
                return Err(format!("引脚 {} 被重复使用", pins[i]));
            }
        }
    }
    
    // 检查引脚编号是否有效（根据实际可用的 GPIO 引脚）
    let valid_pins = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 21];
    
    for &pin in &pins {
        if !valid_pins.contains(&pin) {
            return Err(format!("引脚 {pin} 不是有效的 GPIO 引脚。有效引脚: {valid_pins:?}"));
        }
    }
    
    Ok(())
}


