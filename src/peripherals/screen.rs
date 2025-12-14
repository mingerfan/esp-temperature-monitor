use anyhow::Result;
use embedded_hal::spi::SpiDevice;
use esp_idf_svc::hal::gpio::{self, AnyIOPin, InputOutput, PinDriver};
use esp_idf_svc::hal::spi::{SPI2, SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use ssd1306::{prelude::*, Ssd1306};
use ssd1306::mode::DisplayConfig;
use embedded_graphics::{
    mono_font::{iso_8859_1::FONT_6X10, iso_8859_1::FONT_9X18_BOLD, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

type IOPinDriver = PinDriver<'static, gpio::AnyIOPin, InputOutput>;

pub fn to_point(x: i32, y: i32) -> Point {
    Point::new(x, y)
}


/// Screen Builder，用于封装 SPI 和屏幕初始化
pub struct ScreenBuilder;

impl ScreenBuilder {

    /// 从 SPI 外设和 GPIO pins 创建 Screen 实例
    /// 
    /// 默认推荐引脚：
    /// - GPIO2: SPI SCK
    /// - GPIO0: SPI MOSI
    /// - GPIO18: SPI CS
    /// - GPIO12: DC (数据/命令)
    /// 
    /// # Arguments
    /// * `spi2` - SPI2 外设
    /// * `sck` - SPI SCK 引脚
    /// * `mosi` - SPI MOSI 引脚
    /// * `cs` - SPI CS 片选引脚
    /// * `dc` - 屏幕 DC (数据/命令) 引脚
    /// 
    /// # Returns
    /// * `Result<Screen>` - 成功返回 Screen 实例
    pub fn with_pins(
        spi2: SPI2,
        sck: impl Into<AnyIOPin>,
        mosi: impl Into<AnyIOPin>,
        cs: impl Into<AnyIOPin>,
        dc: impl Into<AnyIOPin>,
    ) -> Result<Screen<SpiDeviceDriver<'static, SpiDriver<'static>>>> {
        // 转换为 AnyIOPin
        let sck: AnyIOPin = sck.into();
        let mosi: AnyIOPin = mosi.into();
        let cs: AnyIOPin = cs.into();
        let dc: AnyIOPin = dc.into();

        // 配置 SPI 驱动
        let driver_config = SpiDriverConfig::new();
        let config = SpiConfig::new().write_only(true);

        // 创建 SPI 驱动
        let spi = SpiDriver::new(
            spi2,
            sck,
            mosi,
            Option::<AnyIOPin>::None,
            &driver_config,
        )?;

        // 创建 SPI 设备驱动
        let spi_device = SpiDeviceDriver::new(spi, Some(cs), &config)?;

        // 创建屏幕
        Screen::new(spi_device, dc)
    }

}

pub struct Screen<SPI: SpiDevice> {
    driver: Ssd1306<SPIInterface<SPI, IOPinDriver>, DisplaySize128x64, ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>>,
}

impl<SPI: SpiDevice> Screen<SPI> {
    pub fn new(spi: SPI, dc_io: gpio::AnyIOPin) -> Result<Self> {
        let dc_io = PinDriver::input_output(dc_io)?;

        let interface = SPIInterface::new(spi, dc_io);
        let size = DisplaySize128x64;
        let rotation = DisplayRotation::Rotate0;
        let mut driver = Ssd1306::new(interface, size, rotation).into_buffered_graphics_mode();
        
        driver.init().map_err(|_| anyhow::anyhow!("Screen init failed"))?;

        // 初始化屏幕代码
        Ok(Self { driver})
    }

    // pub fn draw_example(&mut self) -> Result<()> {
    //     // 画一个圆
    //     Circle::new(Point::new(64, 32), 30)
    //         .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    //         .draw(&mut self.driver)
    //         .map_err(|_| anyhow::anyhow!("Circle draw failed"))?;

    //     // 显示文本
    //     let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    //     Text::new("Hello!", Point::new(10, 10), style)
    //         .draw(&mut self.driver)
    //         .map_err(|_| anyhow::anyhow!("Text draw failed"))?;

    //     // 刷新到屏幕
    //     self.driver.flush().map_err(|_| anyhow::anyhow!("Screen flush failed"))?;
    //     Ok(())
    // }

    // 每次绘制后需要调用 flush 将缓冲区内容显示到屏幕上
    pub fn flush(&mut self) -> Result<()> {
        self.driver.flush().map_err(|_| anyhow::anyhow!("Screen flush failed"))?;
        Ok(())
    }

    // 清理屏幕内容
    pub fn clear(&mut self) -> Result<()> {
        self.driver.clear(BinaryColor::Off).map_err(|_| anyhow::anyhow!("Screen clear failed"))?;
        Ok(())
    }

    pub fn draw_text(&mut self, text: &str, position: Point) -> Result<()> {
        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        Text::new(text, position, style)
            .draw(&mut self.driver)
            .map_err(|_| anyhow::anyhow!("Text draw failed"))?;
        Ok(())
    }

    pub fn draw_text_big(&mut self, text: &str, position: Point) -> Result<()> {
        let style = MonoTextStyle::new(&FONT_9X18_BOLD, BinaryColor::On);
        Text::new(text, position, style)
            .draw(&mut self.driver)
            .map_err(|_| anyhow::anyhow!("Text draw failed"))?;
        Ok(())
    }
}
