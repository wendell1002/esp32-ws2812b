# WS2812B

A library to drive the WS2812B LED.

# Play one LED

```rust
...

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let delay = Delay::new();
    let mut r = WS2812B::new(peripherals.RMT, 80, peripherals.GPIO8).unwrap();

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

...
```

# Build for different esp32 targets

```BASH
cargo run --features esp32c3
```
