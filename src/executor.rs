use core::{
    pin::Pin,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

use futures::Future;

use crate::container::Container;

static VTABLE: RawWakerVTable = {
    unsafe fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &VTABLE)
    }
    unsafe fn wake(p: *const ()) {
        wake_by_ref(p)
    }
    unsafe fn wake_by_ref(p: *const ()) {
        let p = p as *mut () as *mut bool;
        *p = true;
    }
    unsafe fn drop(_: *const ()) {
        // no-op
    }

    RawWakerVTable::new(clone, wake, wake_by_ref, drop)
};

pub struct Executor<'a> {
    tasks: Container<Pin<&'a mut dyn Future<Output = ()>>>,
    woken: [bool; 4],
}

impl<'a> Executor<'a> {
    pub fn new() -> Executor<'a> {
        Executor {
            tasks: Container::new(),
            woken: [false; 4],
        }
    }

    #[allow(unused)]
    pub fn execute(task: impl Future<Output = ()>) {
        let mut exec = Executor::new();
        let mut f = async {
            task.await;
        };
        let f = &mut f;
        exec.spawn(f);
        exec.run();
    }

    #[allow(unused)]
    pub fn execute2(task1: impl Future<Output = ()>, task2: impl Future<Output = ()>) {
        let mut exec = Executor::new();

        let mut f1 = async {
            task1.await;
        };
        let f1 = &mut f1;
        exec.spawn(f1);

        let mut f2 = async {
            task2.await;
        };
        let f2 = &mut f2;
        exec.spawn(f2);

        exec.run();
    }

    pub fn spawn(&mut self, f: &'a mut dyn Future<Output = ()>) {
        let f = unsafe { Pin::new_unchecked(f) };
        let i = self.tasks.push(f);
        self.woken[i] = true;
    }

    pub fn run(&mut self) {
        let mut done = false;

        while !done {
            riscv::interrupt::free(|_| {
                let mut to_remove = Container::<usize>::new();
                let mut iter = self.tasks.iter();
                while let (i, Some(f)) = iter.next() {
                    if self.woken[i] {
                        self.woken[i] = false;

                        let woken_ptr = &self.woken[i] as *const bool as *const ();
                        let my_waker = RawWaker::new(woken_ptr, &VTABLE);
                        let waker = unsafe { Waker::from_raw(my_waker) };
                        let mut ctx = Context::from_waker(&waker);

                        let result = f.as_mut().poll(&mut ctx);

                        if result.is_ready() {
                            to_remove.push(i);
                        }
                    }
                }

                let mut to_remove_iter = to_remove.iter();
                while let (_, Some(i)) = to_remove_iter.next() {
                    self.tasks.remove(*i);
                    self.woken[*i] = false;
                }

                done = self.tasks.size() == 0;
            });
        }
    }
}
