use anyhow::Result;
use embedded_hal::spi::SpiDevice;
use esp_idf_svc::hal::gpio::{self, InputOutput, PinDriver};
use ssd1306::{prelude::*, Ssd1306};
use ssd1306::mode::DisplayConfig;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::Text,
};

type IOPinDriver = PinDriver<'static, gpio::AnyIOPin, InputOutput>;

struct Screen<SPI: SpiDevice> {
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

    pub fn draw_example(&mut self) -> Result<()> {
        // 画一个圆
        Circle::new(Point::new(64, 32), 30)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.driver)
            .map_err(|_| anyhow::anyhow!("Circle draw failed"))?;

        // 显示文本
        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        Text::new("Hello!", Point::new(10, 10), style)
            .draw(&mut self.driver)
            .map_err(|_| anyhow::anyhow!("Text draw failed"))?;

        // 刷新到屏幕
        self.driver.flush().map_err(|_| anyhow::anyhow!("Screen flush failed"))?;
        Ok(())
    }
}
