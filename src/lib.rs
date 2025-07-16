#![no_std]

//! # WS2812B
//!
//! A library to drive the WS2812B LED.
//!
//! # Play one LED
//! ```rust
//! ...
//!
//! let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
//! let peripherals: esp_hal::peripherals::Peripherals = esp_hal::init(config);
//!
//! let mut r = WS2812B::new(peripherals.RMT, 80, peripherals.GPIO8).unwrap();
//!
//! r = r.send(colors::ALICE_BLUE, 10, 1).expect("Failed to dispatch!");
//!
//! ...
//! ```
//!
//! # Build for different esp32 targets
//!
//! ```BASH
//! cargo run --features esp32c6
//! ```

//! Module documentation
//!
//! # Fade multiple LEDs
//! ```rust
//!
//! use esp32_ws2812b::WS2812B;
//! let mut r = WS2812B::new(peripherals.RMT, 80, peripherals.GPIO8).unwrap();
//!
//! loop {
//!   r = r.fade(123)?;
//! }
//!
//! ```
//!
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin, Pin};
use esp_hal::peripherals::RMT;
use esp_hal::rmt::{Channel, PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator};
use esp_hal::time::Rate;
use esp_hal::Blocking;
use smart_leds::{RGB, RGB8, RGBA};

const BRG_MAX_NUM_OF_LEDS: usize = 256;
const BRG_PACKET_SIZE: usize = 24;

#[derive(Debug)]
pub enum Error {
    TooManyLeds,
    RmtError(esp_hal::rmt::Error),
}

// 'From' trait
impl From<esp_hal::rmt::Error> for Error {
    fn from(err: esp_hal::rmt::Error) -> Self {
        Error::RmtError(err)
    }
}

pub struct WS2812B {
    ch: Channel<Blocking, 0>,
}

impl WS2812B {
    /// Create a WS2812B instance with RGB(0, 0, 0)
    ///
    /// Here's an example:
    ///
    /// ```
    /// use ws2812_rgb::ws2812b::WS2812B;
    /// let mut led = WS2812B::new(peripherals.RMT, 80, peripherals.GPIO8)?;
    /// ```
    pub fn new(rmt: RMT, freq_mhz: u32, gpio: impl OutputPin) -> Result<Self, Error> {
        let rmt = Rmt::new(rmt, Rate::from_mhz(freq_mhz))?;
        let output: Output<'_> = Output::new(gpio, Level::High, OutputConfig::default());
        let tick_rate: u32 = (freq_mhz * 5) / 100; // 50 ns tick!
        let channel = rmt.channel0.configure(
            output,
            TxChannelConfig::default().with_clk_divider(tick_rate as u8),
        )?;

        Ok(WS2812B { ch: channel })
    }

    pub fn send(self, rgb: RGB8, brightness: u8, num: usize) -> Result<Self, Error> {
        let rgb = RGB8 {
            r: (rgb.r as u16 * (brightness as u16 + 1) / 256) as u8,
            g: (rgb.g as u16 * (brightness as u16 + 1) / 256) as u8,
            b: (rgb.b as u16 * (brightness as u16 + 1) / 256) as u8,
        };
        self.write(rgb, num)
    }

    pub fn write(mut self, rgb: RGB8, num: usize) -> Result<Self, Error> {
        if num >= BRG_MAX_NUM_OF_LEDS - 1 {
            return Err(Error::TooManyLeds);
        }
        // Create final stream of data.
        let mut data: [u32; BRG_PACKET_SIZE * BRG_MAX_NUM_OF_LEDS] =
            [u32::default(); BRG_PACKET_SIZE * BRG_MAX_NUM_OF_LEDS];

        // Create RGB packet. (Always the same for now.)
        let packet = self.build_packet(rgb.g, rgb.r, rgb.b);

        for i in 0..num {
            let index = i * BRG_PACKET_SIZE;
            data[index..(index + BRG_PACKET_SIZE)].copy_from_slice(&packet);
        }

        data[num * BRG_PACKET_SIZE] = PulseCode::empty();
        // Slice one index extra to fit the `PulseCode::empty()`;
        self = self.dispatch(&data[0..((num * BRG_PACKET_SIZE) + 1)])?;

        Ok(self)
    }

    /// This function will play a fade animation based on the RGB colors.
    ///
    /// Note that the function uses `delay_millis`, and thus, it blocks for
    /// a period of time: 5 ms.
    pub fn fade(mut self, rgb: RGB8, num: usize) -> Result<Self, Error> {
        let delay = Delay::new();
        let mut rgb = RGB8 {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        };
        let setters = [Self::set_blue, Self::set_green, Self::set_red];

        for setter in setters {
            for color in (0..255).chain((0..255).rev()) {
                self = self.write(rgb, num)?;
                // setter(&mut self, color);
                setter(&self, &mut rgb, color);

                delay.delay_millis(5);
            }
        }
        Ok(self)
    }

    fn set_red(&self, rgb: &mut RGB8, color: u8) {
        rgb.r = color;
    }
    fn set_green(&self, rgb: &mut RGB8, color: u8) {
        rgb.g = color;
    }
    fn set_blue(&self, rgb: &mut RGB8, color: u8) {
        rgb.b = color;
    }

    fn dispatch(mut self, data: &[u32]) -> Result<Self, Error> {
        let transaction = self.ch.transmit(&data)?;
        self.ch = transaction.wait().map_err(|tup_e| tup_e.0)?;

        Ok(self)
    }

    // in ns: 800/450
    fn get_bit_one(&self) -> u32 {
        PulseCode::new(Level::High, 16, Level::Low, 9)
    }

    // in ns: 400/850
    fn get_bit_zero(&self) -> u32 {
        PulseCode::new(Level::High, 8, Level::Low, 17)
    }

    fn build_packet(&self, r: u8, g: u8, b: u8) -> [u32; BRG_PACKET_SIZE] {
        let mut data: [u32; BRG_PACKET_SIZE] = [0; BRG_PACKET_SIZE];
        let mut index: usize = 0;

        for byte in &[g, r, b] {
            for bit_index in (0..8).rev() {
                if (*byte >> bit_index) & 0x01 == 0x01 {
                    data[index] = self.get_bit_one();
                } else {
                    data[index] = self.get_bit_zero();
                }
                index += 1;
            }
        }

        data
    }
}
