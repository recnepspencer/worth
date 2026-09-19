use super::*;
use std::sync::atomic::AtomicUsize;
use std::task::Wake;

#[derive(Default)]
struct WakeCount(AtomicUsize);

impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn dropped_waiter_does_not_retain_or_remove_a_live_peer_registration() {
    let source = WorthQueryCancellationSource::new();
    let token = source.token();
    let dropped_count = Arc::new(WakeCount::default());
    let live_count = Arc::new(WakeCount::default());
    let dropped_waker = Waker::from(Arc::clone(&dropped_count));
    let live_waker = Waker::from(Arc::clone(&live_count));
    let mut dropped = Box::pin(token.cancelled());
    let mut live = Box::pin(token.cancelled());
    assert!(dropped
        .as_mut()
        .poll(&mut Context::from_waker(&dropped_waker))
        .is_pending());
    assert!(live
        .as_mut()
        .poll(&mut Context::from_waker(&live_waker))
        .is_pending());
    drop(dropped);
    assert_eq!(source.state.waiters.lock().unwrap().len(), 1);
    assert_eq!(Arc::strong_count(&dropped_count), 2);
    source.cancel();
    assert_eq!(dropped_count.0.load(Ordering::SeqCst), 0);
    assert_eq!(live_count.0.load(Ordering::SeqCst), 1);
    assert!(live
        .as_mut()
        .poll(&mut Context::from_waker(&live_waker))
        .is_ready());
    assert!(source.state.waiters.lock().unwrap().is_empty());
}
