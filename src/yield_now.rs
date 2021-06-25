use core::task::Poll;
use futures::Future;

struct YieldNow {
    yield_now: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.yield_now {
            self.get_mut().yield_now = false;
            cx.waker().clone().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub async fn yield_now() {
    let yield_now = YieldNow { yield_now: true };
    yield_now.await;
}
