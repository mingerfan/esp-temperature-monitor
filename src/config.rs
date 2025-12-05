//! 外设配置模块
//! 
//! 提供引脚和外设的配置定义和管理器

pub mod pins;
pub mod manager;

// 重新导出常用类型
pub use manager::PeripheralManager;