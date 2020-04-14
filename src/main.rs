//! This example is for the STM32F103 "Black Pill" board using a 4 wire interface to the display on
//! SPI1.
//!
//! Wiring connections are as follows
//!
//! ```
//! GND -> GND
//! 3V3 -> VCC
//! PA5 -> SCL / clk
//! PA7 -> SDA / din
//! PB0 -> RST / res
//! PB1 -> D/C
//! ```
//!

#![no_std]
#![no_main]

mod display;
mod temperature;

use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use panic_semihosting as _;

use hal::{
    delay::Delay,
    interrupt,
    prelude::*,
    rcc::Clocks,
    stm32,
    timer::{Event, Timer},
};
use stm32f4xx_hal as hal;

static TEMP_UPDATE_REQUIRED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static ELAPSED_MS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0u32));
static TIMER_TIM2: Mutex<RefCell<Option<Timer<stm32::TIM2>>>> = Mutex::new(RefCell::new(None));

fn timer2_init(clocks: Clocks, tim2: stm32f4xx_hal::stm32::TIM2) {
    let mut timer = Timer::tim2(tim2, 2.hz(), clocks);
    timer.listen(Event::TimeOut);
    free(|cs| {
        TIMER_TIM2.borrow(cs).replace(Some(timer));
    });

    // enable
    unsafe {
        stm32::NVIC::unmask(hal::stm32::Interrupt::TIM2);
    }

    // Enable interrupts
    stm32::NVIC::unpend(hal::stm32::Interrupt::TIM2);
}

fn temp_update_required() -> bool {
    free(|cs| {
        let cell = TEMP_UPDATE_REQUIRED.borrow(cs);
        let res = cell.get();
        if res {
            cell.replace(false);
        }
        res
    })
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(25.mhz())
        .sysclk(100.mhz())
        .pclk1(25.mhz())
        .pclk2(25.mhz())
        .freeze();

    let mut delay = Delay::new(cp.SYST, clocks);

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    // Display pins
    let sck = gpioa.pa5.into_alternate_af5();
    let miso = gpioa.pa6.into_alternate_af5();
    let mosi = gpioa.pa7.into_alternate_af5();
    let rst = gpiob.pb0.into_push_pull_output();
    let dc = gpiob.pb1.into_push_pull_output();

    let mut disp = display::Display::new(dp.SPI1, clocks, &mut delay, (sck, miso, mosi), rst, dc);

    disp.draw_rust(true);

    // OneWire pin
    let mut one = gpiob.pb6.into_open_drain_output().downgrade();
    let mut temp = temperature::Temperature::new(&mut delay, &mut one);

    // Timer initialization
    timer2_init(clocks, dp.TIM2);

    if !temp.has_device() {
        disp.draw_text("NF", true);
    }

    loop {
        if temp_update_required() {
            if temp.has_device() {
                let t = temp.get(&mut delay);
                disp.draw_temp(t.0, t.1, true);
            } else {
                disp.draw_rust(true);
            }
            let elapsed = free(|cs| ELAPSED_MS.borrow(cs).get());
            disp.draw_num(elapsed, false);
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some(ref mut tim2) = TIMER_TIM2.borrow(cs).borrow_mut().deref_mut() {
            tim2.clear_interrupt(Event::TimeOut);
        }

        let cell = ELAPSED_MS.borrow(cs);
        let val = cell.get();
        cell.replace(val + 1);

        let cell = TEMP_UPDATE_REQUIRED.borrow(cs);
        cell.replace(true);
    });
}
