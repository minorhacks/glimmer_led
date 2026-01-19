//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod ws2812;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIO0, SPI0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

use crate::ws2812::Ws2812;

use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Stack, StackResources};
use embassy_net_wiznet::chip::W5500;
use embassy_net_wiznet::{Device, Runner, State as WiznetState};
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

type MySpi = ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static>, Delay>;
type MyWiznetRunner = Runner<'static, W5500, MySpi, Input<'static>, Output<'static>>;
type MyNetRunner = embassy_net::Runner<'static, Device<'static>>;

#[embassy_executor::task]
async fn eth_task(runner: MyWiznetRunner) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: MyNetRunner) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn udp_task(stack: Stack<'static>) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut buf = [0; 4096];

    loop {
        let mut socket = UdpSocket::new(
            stack,
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );
        socket.bind(1234).unwrap();

        info!("Listening on UDP:1234");

        loop {
            match socket.recv_from(&mut buf).await {
                Ok((n, ep)) => {
                    info!("Received {} bytes from {}", n, ep);
                    if let Ok(s) = core::str::from_utf8(&buf[..n]) {
                        info!("Payload: {}", s);
                    }
                }
                Err(e) => {
                    warn!("UDP recv error: {:?}", e);
                }
            }
        }
    }
}

static WIZNET_STATE: StaticCell<WiznetState<8, 8>> = StaticCell::new();
static STACK_RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start WS2812 and Ethernet");
    let p = embassy_rp::init(Default::default());

    // WS2812B driver initialization
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);
    let mut ws2812 = Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_2);

    // Ethernet Initialization
    let mut spi_cfg = Config::default();
    spi_cfg.frequency = 50_000_000; // 50 MHz
    let spi = Spi::new(
        p.SPI0, p.PIN_18, p.PIN_19, p.PIN_16, p.DMA_CH1, p.DMA_CH2, spi_cfg,
    );
    let cs = Output::new(p.PIN_17, Level::High);
    let spi_dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();

    let w5500_int = Input::new(p.PIN_21, Pull::Up); // INT
    let w5500_rst = Output::new(p.PIN_20, Level::High); // RST

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x02]; // Example MAC
    let state = WIZNET_STATE.init(WiznetState::new());

    let (device, runner) = embassy_net_wiznet::new(mac_addr, state, spi_dev, w5500_int, w5500_rst)
        .await
        .unwrap();

    unwrap!(spawner.spawn(eth_task(runner)));

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // Should be random in production

    // Init Network Stack
    let config = embassy_net::Config::dhcpv4(Default::default());
    let stack_resources = STACK_RESOURCES.init(StackResources::new());
    let (stack, runner) = embassy_net::new(device, config, stack_resources, seed);

    unwrap!(spawner.spawn(net_task(runner)));
    unwrap!(spawner.spawn(udp_task(stack)));

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
            colors[i * 3 + 1] = 0; // R
            colors[i * 3 + 2] = 0; // B

            ws2812.write(&colors).await;
            Timer::after_millis(100).await;
        }
    }
}
