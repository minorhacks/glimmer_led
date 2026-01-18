use embassy_rp::{clocks, dma, into_ref, pio, Peripheral, PeripheralRef};
use fixed::traits::ToFixed;

pub struct Ws2812<'d, PIO: pio::Instance, const SM: usize, DMA> {
    sm: pio::StateMachine<'d, PIO, SM>,
    dma: PeripheralRef<'d, DMA>,
}

impl<'d, PIO: pio::Instance, const SM: usize, DMA> Ws2812<'d, PIO, SM, DMA>
where
    DMA: dma::Channel,
{
    pub fn new(
        pio: &mut pio::Common<'d, PIO>,
        mut sm: pio::StateMachine<'d, PIO, SM>,
        dma: impl Peripheral<P = DMA> + 'd,
        pin: impl pio::PioPin,
    ) -> Self {
        into_ref!(dma);

        let pin = pio.make_pio_pin(pin);
        sm.set_pin_dirs(pio::Direction::Out, &[&pin]);

        // Standard WS2812B protocol:
        // 800kHz data rate
        // T0H: 0.4us, T0L: 0.85us
        // T1H: 0.8us, T1L: 0.45us
        // Error of +/- 150ns allowed.

        let prg = pio_proc::pio_asm!(
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

        let mut cfg = pio::Config::default();

        // Pin configuration
        cfg.use_program(&relocated, &[&pin]);
        cfg.shift_in.auto_fill = false; // We are outputting only
        cfg.shift_out.auto_fill = true; // Auto pull from FIFO
        cfg.shift_out.threshold = 8;    // 8 bit threshold (byte by byte)
        cfg.shift_out.direction = pio::ShiftDirection::Left; // MSB first
        cfg.fifo_join = pio::FifoJoin::TxOnly;

        // Clock configuration
        // Standard: 800kHz = 1.25us per bit.
        // With 10 cycles per bit, clock = 8MHz.
        
        let freq = 8_000_000;
        cfg.clock_divider = (clocks::clk_sys_freq() / freq).to_fixed();

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            sm,
            dma: dma.into_ref(),
        }
    }

    pub async fn write(&mut self, colors: &[u8]) {
        self.sm.tx().dma_push(self.dma.reborrow(), colors).await;
    }
}
