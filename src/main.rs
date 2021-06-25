#![no_std]
#![no_main]

use async_timer::{async_wait, timer_init};
use bl602_hal as hal;
use core::cell::RefCell;
use core::fmt::Write;
use executor::Executor;
use futures::FutureExt;
use hal::{
    clock::{Strict, SysclkFreq, UART_PLL_FREQ},
    pac,
    prelude::*,
    serial::*,
    timer::TimerExt,
};
use panic_halt as _;
use riscv::interrupt::Mutex;

use crate::yield_now::yield_now;

mod async_timer;
mod container;
mod executor;
mod yield_now;

#[riscv_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut parts = dp.GLB.split();

    // Set up all the clocks we need
    let clocks = Strict::new()
        .use_pll(40_000_000u32.Hz())
        .sys_clk(SysclkFreq::Pll160Mhz)
        .uart_clk(UART_PLL_FREQ.Hz())
        .freeze(&mut parts.clk_cfg);

    // Set up uart output. Since this microcontroller has a pin matrix,
    // we need to set up both the pins and the muxs
    let pin16 = parts.pin16.into_uart_sig0();
    let pin7 = parts.pin7.into_uart_sig7();
    let mux0 = parts.uart_mux0.into_uart0_tx();
    let mux7 = parts.uart_mux7.into_uart0_rx();

    // Configure our UART to 115200Baud, and use the pins we configured above
    let mut serial = Serial::uart0(
        dp.UART,
        Config::default().baudrate(115_200.Bd()),
        ((pin16, mux0), (pin7, mux7)),
        clocks,
    );
    write!(serial, "start\r\n").ok();

    let serial = Mutex::new(RefCell::new(serial));

    let timers = dp.TIMER.split();
    timer_init(timers.channel0);

    Executor::execute2(
        async {
            loop {
                async_wait(2)
                    .then(|_| async {
                        riscv::interrupt::free(|cs| {
                            let serial = serial.borrow(cs);
                            write!(*serial.borrow_mut(), "Hello Rust\r\n").ok();
                        });
                    })
                    .await;

                yield_now().await;
            }
        },
        async {
            loop {
                async_wait(1)
                    .then(|_| async {
                        riscv::interrupt::free(|cs| {
                            let serial = serial.borrow(cs);
                            write!(*serial.borrow_mut(), "Hello World\r\n").ok();
                        });
                    })
                    .await;

                yield_now().await;
            }
        },
    );

    loop {}
}
