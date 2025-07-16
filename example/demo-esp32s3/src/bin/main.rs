#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp32_ws2812b::WS2812B;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

use log::info;
use smart_leds::colors;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let delay = Delay::new();
    let mut r = WS2812B::new(peripherals.RMT, 80, peripherals.GPIO48).unwrap();

    loop {
        r = r.fade(colors::BLUE, 1).expect("Failed to dispatch!");
        // r = r
        //     .send(colors::ALICE_BLUE, 10, 1)
        //     .expect("Failed to dispatch!");
        // delay.delay_millis(500);
        // r = r
        //     .send(colors::CHOCOLATE, 10, 1)
        //     .expect("Failed to dispatch!");
        // delay.delay_millis(500);
        // r = r.send(colors::OLIVE, 10, 1).expect("Failed to dispatch!");
        delay.delay_millis(500);
    }
}
