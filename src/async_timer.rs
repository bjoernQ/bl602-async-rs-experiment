use core::mem::MaybeUninit;
use core::task::{Poll, Waker};

use bl602_hal as hal;
use bl602_hal::timer::{ClockSource, TimerChannel0};
use embedded_time::duration::Milliseconds;
use futures::Future;
use hal::interrupts::TrapFrame;
use hal::prelude::Extensions;
use hal::timer::ConfiguredTimerChannel0;

use crate::container::Container;

static mut CH0: MaybeUninit<ConfiguredTimerChannel0> = MaybeUninit::uninit();

static mut WAKERS: Container<(u32, Waker)> = Container::new();
static mut WAKER_HANDLE: u32 = 0;

pub fn timer_init(channel0: TimerChannel0) {
    let ch0 = channel0.set_clock_source(ClockSource::Clock1Khz, 1_000u32.Hz());
    ch0.enable_match0_interrupt();
    ch0.set_preload_value(Milliseconds::new(0));
    ch0.set_preload(hal::timer::Preload::PreloadMatchComparator0);
    ch0.set_match0(Milliseconds::new(500u32));

    hal::interrupts::enable_interrupt(hal::interrupts::Interrupt::TimerCh0);
    unsafe {
        *(CH0.as_mut_ptr()) = ch0;
    }

    get_ch0().enable(); // start timer for tasks

    unsafe {
        riscv::interrupt::enable();
    }
}

#[allow(non_snake_case)]
#[no_mangle]
fn TimerCh0(_trap_frame: &mut TrapFrame) {
    get_ch0().clear_match0_interrupt();

    let mut wakers_iter = unsafe { WAKERS.iter() };
    while let (_, Some((_, waker))) = wakers_iter.next() {
        waker.wake_by_ref();
    }
}

fn get_ch0() -> &'static mut ConfiguredTimerChannel0 {
    unsafe { &mut *CH0.as_mut_ptr() }
}

struct TimerFuture {
    target: u32,
    current: u32,
    handle: u32,
}

impl TimerFuture {
    fn new(target: u32) -> TimerFuture {
        let handle = unsafe {
            WAKER_HANDLE = WAKER_HANDLE.overflowing_add(1).0;
            WAKER_HANDLE
        };

        TimerFuture {
            target,
            current: 0,
            handle,
        }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.current >= self.target {
            Poll::Ready(())
        } else {
            let this = self.get_mut();
            this.current += 1;

            let waker = cx.waker().clone();
            let handle = this.handle;

            unsafe {
                let mut to_remove: Option<usize> = None;
                let mut wakers_iter = WAKERS.iter();

                while let (index, Some((hndl, _))) = wakers_iter.next() {
                    if handle == *hndl {
                        to_remove = Some(index);
                    }
                }

                if let Some(i) = to_remove {
                    WAKERS.remove(i);
                }
                WAKERS.push((handle, waker));
            }

            Poll::Pending
        }
    }
}

pub fn async_wait(ticks: u32) -> impl Future<Output = ()> {
    TimerFuture::new(ticks)
}
