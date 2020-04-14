use core::convert::Infallible;

use hal::{
    delay::Delay,
    prelude::*,
    rcc::Clocks,
    spi::{Mode, Phase, Polarity, Spi},
    stm32::SPI1,
};
use stm32f4xx_hal as hal;

use embedded_hal::digital::v2::OutputPin;

use embedded_graphics::{
    fonts::{Font, Font12x16, Font6x8, Text},
    geometry::Point,
    image::{Image, ImageRawLE},
    pixelcolor::Rgb565,
    prelude::*,
    style::TextStyleBuilder,
};

use ssd1331::{DisplayRotation::Rotate0, Ssd1331};

// TODO: use core formatting?
use heapless::consts::*;
use heapless::String;

pub struct Display<PINS, DC> {
    disp: Ssd1331<stm32f4xx_hal::spi::Spi<SPI1, PINS>, DC>,
}

impl<PINS, DC> Display<PINS, DC> {
    pub fn new<RST>(
        spi1: SPI1,
        clocks: Clocks,
        delay: &mut Delay,
        pins: PINS,
        mut rst: RST,
        dc: DC,
    ) -> Self
    where
        PINS: stm32f4xx_hal::spi::Pins<SPI1>,
        DC: OutputPin<Error = Infallible>,
        RST: OutputPin<Error = Infallible>,
    {
        let spi = Spi::spi1(
            spi1,
            pins,
            Mode {
                polarity: Polarity::IdleHigh,
                phase: Phase::CaptureOnSecondTransition,
            },
            8.mhz().into(),
            clocks,
        );

        let mut disp = Ssd1331::new(spi, dc, Rotate0);
        disp.reset(&mut rst, delay).unwrap();
        delay.delay_ms(1000_u32);
        disp.init().unwrap();
        disp.clear();
        disp.flush().unwrap();
        disp.clear();
        disp.flush().unwrap();
        Display { disp }
    }

    pub fn draw_rust(&mut self, clear: bool)
    where
        DC: OutputPin<Error = Infallible>,
    {
        if clear {
            self.disp.clear();
        }

        // Loads an 86x64px image encoded in LE (Little Endian) format. This image is a 16BPP image of
        // the Rust mascot, Ferris.
        let im = ImageRawLE::new(include_bytes!("./ferris.raw"), 86, 64);
        Image::new(&im, Point::new((96 - 86) / 2, 0))
            .draw(&mut self.disp)
            .unwrap();

        let _ = self.disp.flush();
    }

    fn draw_text_xyc<F>(&mut self, text: &str, font: F, color: Rgb565, x: i32, y: i32, clear: bool)
    where
        DC: OutputPin<Error = Infallible>,
        F: Font + Clone + Copy,
    {
        if clear {
            self.disp.clear();
        }

        let text_style = TextStyleBuilder::new(font).text_color(color).build();

        Text::new(text, Point::new(x, y))
            .into_styled(text_style)
            .draw(&mut self.disp)
            .unwrap();
        let _ = self.disp.flush();
    }

    pub fn draw_text(&mut self, text: &str, clear: bool)
    where
        DC: OutputPin<Error = Infallible>,
    {
        self.draw_text_xyc(text, Font6x8, Rgb565::WHITE, 0, 0, clear);
    }

    pub fn draw_num(&mut self, val: u32, clear: bool)
    where
        DC: OutputPin<Error = Infallible>,
    {
        let s: String<U10> = val.into();
        self.draw_text_xyc(s.as_str(), Font12x16, Rgb565::RED, 0, 25, clear);
    }

    pub fn draw_temp(&mut self, val1: u16, val2: u16, clear: bool)
    where
        DC: OutputPin<Error = Infallible>,
    {
        let mut s: String<U10> = val1.into();
        s.push_str(".").unwrap();
        let s2: String<U5> = val2.into();
        s.push_str(&s2[0..1]).unwrap(); // use only one digit
        s.push_str(" °C").unwrap();
        self.draw_text_xyc(s.as_str(), Font12x16, Rgb565::WHITE, 0, 0, clear);
    }
}
