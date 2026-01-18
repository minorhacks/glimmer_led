//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod ws2812;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use crate::ws2812::Ws2812;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start WS2812");
    let p = embassy_rp::init(Default::default());
    
    // WS2812B driver initialization
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);
    let mut ws2812 = Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_2);

    // Test pattern: Red, Green, Blue, White moving
    let mut colors = [0u8; 3 * 10]; // 10 LEDs

    loop {
        // Simple chasing pattern
        for i in 0..10 {
           // Clear
           colors.fill(0);
           // Set one LED to Green (GRB format for WS2812B usually)
           // Actually format depends on specific strip, usually GRB.
           // Let's set Green
           colors[i * 3] = 20; // G
           colors[i * 3 + 1] = 0;  // R
           colors[i * 3 + 2] = 0;  // B
           
           ws2812.write(&colors).await;
           Timer::after_millis(100).await;
        }
    }
}
