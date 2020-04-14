use core::convert::Infallible;

use hal::{delay::Delay, prelude::*};
use stm32f4xx_hal as hal;

use onewire::{ds18b20, ds18b20::DS18B20, DeviceSearch, OneWire, OpenDrainOutput};

pub struct Temperature<'a> {
    wire: OneWire<'a, Infallible>,
    device: Option<DS18B20>,
}

impl<'a> Temperature<'a> {
    pub fn new(delay: &mut Delay, output: &'a mut dyn OpenDrainOutput<Infallible>) -> Self {
        let mut wire = OneWire::new(output, false);
        wire.reset(delay).unwrap();
        let device = Self::search_ds18b20_device(&mut wire, delay);
        Temperature { wire, device }
    }

    pub fn has_device(&self) -> bool {
        !self.device.is_none()
    }

    pub fn get(&mut self, delay: &mut Delay) -> (u16, u16) {
        if let Some(ref mut ds18b20) = self.device {
            let resolution = ds18b20.measure_temperature(&mut self.wire, delay).unwrap();
            delay.delay_ms(resolution.time_ms());
            let temperature = ds18b20.read_temperature(&mut self.wire, delay).unwrap();
            let temperature2 = (temperature % 16) * 10;
            return (temperature / 16, temperature2 / 16);
        }
        (0, 0)
    }

    fn search_ds18b20_device(
        wire: &mut OneWire<core::convert::Infallible>,
        delay: &mut Delay,
    ) -> Option<DS18B20> {
        let mut search = DeviceSearch::new();
        if let Some(device) = wire.search_next(&mut search, delay).unwrap() {
            if device.address[0] == ds18b20::FAMILY_CODE {
                let ds18b20: Result<DS18B20, onewire::Error<()>> = DS18B20::new(device);
                return Some(ds18b20.unwrap());
            }
        }
        None
    }
}
