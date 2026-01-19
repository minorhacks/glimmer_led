use embassy_rp::{clocks, dma, pio as rp_pio, Peri, PeripheralType};
use fixed::traits::ToFixed;

pub struct Ws2812<'d, PIO: rp_pio::Instance, const SM: usize, DMA: PeripheralType> {
    sm: rp_pio::StateMachine<'d, PIO, SM>,
    dma: Peri<'d, DMA>,
}

impl<'d, PIO: rp_pio::Instance, const SM: usize, DMA> Ws2812<'d, PIO, SM, DMA>
where
    DMA: dma::Channel + PeripheralType,
{
    pub fn new(
        pio: &mut rp_pio::Common<'d, PIO>,
        mut sm: rp_pio::StateMachine<'d, PIO, SM>,
        dma: Peri<'d, DMA>,
        pin: Peri<'d, impl rp_pio::PioPin + 'd>,
    ) -> Self {
        let pin = pio.make_pio_pin(pin);
        sm.set_pin_dirs(rp_pio::Direction::Out, &[&pin]);

        // Standard WS2812B protocol:
        // 800kHz data rate
        // T0H: 0.4us, T0L: 0.85us
        // T1H: 0.8us, T1L: 0.45us
        // Error of +/- 150ns allowed.

        let prg = pio::pio_asm!(
            ".side_set 1",
            ".wrap_target",
            "bitloop:",
            "out x, 1        side 0 [2]", // T0L / T1L (start) - always low initially, wait 2
            "jmp !x do_zero  side 1 [1]", // T0H / T1H - drive high. If 0, jump to do_zero (short high). If 1, fall through (long high).
            "do_one:",
            "jmp  bitloop    side 1 [4]", // T1H continued (long high), then loop.
            "do_zero:",
            "nop             side 0 [4]", // T0H -> T0L (short high, then low), then loop.
            ".wrap"
        );

        let relocated = pio.load_program(&prg.program);

        let mut cfg = rp_pio::Config::default();

        // Pin configuration
        cfg.use_program(&relocated, &[&pin]);
        cfg.shift_in.auto_fill = false; // We are outputting only
        cfg.shift_out.auto_fill = true; // Auto pull from FIFO
        cfg.shift_out.threshold = 8; // 8 bit threshold (byte by byte)
        cfg.shift_out.direction = rp_pio::ShiftDirection::Left; // MSB first
        cfg.fifo_join = rp_pio::FifoJoin::TxOnly;

        // Clock configuration
        // Standard: 800kHz = 1.25us per bit.
        // With 10 cycles per bit, clock = 8MHz.

        let freq = 8_000_000;
        cfg.clock_divider = (clocks::clk_sys_freq() / freq).to_fixed();

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self { sm, dma }
    }

    pub async fn write(&mut self, colors: &[u8]) {
        self.sm
            .tx()
            .dma_push(self.dma.reborrow(), colors, false)
            .await;
        embassy_time::Timer::after_micros(60).await;
    }
}
