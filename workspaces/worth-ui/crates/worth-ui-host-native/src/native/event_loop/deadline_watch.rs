//! Keeps timed work progressing while the platform runs its own modal loop.
//!
//! While a window border is dragged, Windows dispatches from a modal loop the
//! event loop does not own. winit then emits no `NewEvents` or `AboutToWait`,
//! and a `WaitUntil` deadline never fires. Frame completion is polled on due
//! ticks of the physical clock, so an in-flight frame would not settle, and no
//! newer extent could present, until the drag ended.
//!
//! The watch posts one wake when an armed deadline passes and the event loop
//! has not reached it, and posts no deadline at or before it again until a
//! wake has progressed timed work. Posted wakes are dispatched by any loop,
//! modal or not.
//! The ordinary loop reaches its deadlines through `WaitUntil` first, so the
//! watch stays silent there. The thread owns no host state and never calls
//! into it; it only posts the wake.

use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use winit::event_loop::EventLoopProxy;

use super::{UiNativeEventLoopApplication, UiNativeEventLoopClient};
use crate::native::readiness::UiNativeApplicationWake;

/// How long past a deadline the event loop may take before the watch wakes it.
const WATCH_SLACK: Duration = Duration::from_millis(4);

pub(super) struct UiNativeDeadlineWatch {
    shared: Arc<(Mutex<UiNativeDeadlineWatchState>, Condvar)>,
    thread: Option<JoinHandle<()>>,
}

#[derive(Default)]
struct UiNativeDeadlineWatchState {
    armed: Option<Instant>,
    /// The deadline of the wake already posted and not yet progressed. A
    /// deadline at or before it is not posted again: the queued wake
    /// progresses it.
    posted: Option<Instant>,
    stopped: bool,
}

impl UiNativeDeadlineWatch {
    pub(super) fn start(proxy: EventLoopProxy<UiNativeApplicationWake>) -> Option<Self> {
        let shared = Arc::new((
            Mutex::new(UiNativeDeadlineWatchState::default()),
            Condvar::new(),
        ));
        let watched = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("worth-ui-deadline-watch".to_owned())
            .spawn(move || {
                watch(&watched, |()| {
                    proxy.send_event(UiNativeApplicationWake).is_ok()
                })
            })
            .ok()?;
        Some(Self {
            shared,
            thread: Some(thread),
        })
    }

    /// Arms the next deadline the event loop waits for, replacing any
    /// earlier one. `progressed` records that the loop has just progressed
    /// its timed work, so a wake posted before then has been consumed.
    pub(super) fn reached(&self, next: Option<Instant>, progressed: bool) {
        let (lock, wake) = &*self.shared;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        if progressed {
            state.posted = None;
        }
        let posted = state.posted;
        let next = next.filter(|next| posted.is_none_or(|posted| *next > posted));
        if next != state.armed {
            state.armed = next;
            wake.notify_one();
        }
    }
}

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    /// Arms the watch with the next deadline the event loop's timed work
    /// waits for: a physical-signal due tick or a presentation retry timeout.
    /// `progressed` is true only after the loop has progressed its timed
    /// work, which an ordinary window event does not do.
    pub(super) fn watch_deadlines(&self, progressed: bool) {
        let shared = self.shared.borrow();
        let physical = shared
            .physical_signal
            .next_due_tick()
            .and_then(|tick| self.physical_clock.deadline(tick));
        let retry = match shared.lifecycle.presentation_retry_wake() {
            Some(crate::native::UiNativePresentationRetryWake::Timeout(deadline)) => Some(deadline),
            Some(crate::native::UiNativePresentationRetryWake::Visibility) | None => None,
        };
        let next = match (physical, retry) {
            (Some(physical), Some(retry)) => Some(physical.min(retry)),
            (physical, retry) => physical.or(retry),
        };
        self.deadline_watch.reached(next, progressed);
    }
}

impl Drop for UiNativeDeadlineWatch {
    fn drop(&mut self) {
        let (lock, wake) = &*self.shared;
        if let Ok(mut state) = lock.lock() {
            state.stopped = true;
            wake.notify_one();
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn watch(shared: &(Mutex<UiNativeDeadlineWatchState>, Condvar), mut post: impl FnMut(()) -> bool) {
    let (lock, wake) = shared;
    let Ok(mut state) = lock.lock() else {
        return;
    };
    loop {
        if state.stopped {
            return;
        }
        let Some(armed) = state.armed else {
            state = match wake.wait(state) {
                Ok(state) => state,
                Err(_) => return,
            };
            continue;
        };
        let due = armed + WATCH_SLACK;
        let now = Instant::now();
        if now < due {
            state = match wake.wait_timeout(state, due - now) {
                Ok((state, _)) => state,
                Err(_) => return,
            };
            continue;
        }
        state.armed = None;
        state.posted = Some(armed);
        drop(state);
        if !post(()) {
            return;
        }
        state = match lock.lock() {
            Ok(state) => state,
            Err(_) => return,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{watch, UiNativeDeadlineWatchState, WATCH_SLACK};
    use std::sync::mpsc;
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::{Duration, Instant};

    fn started() -> (
        Arc<(Mutex<UiNativeDeadlineWatchState>, Condvar)>,
        mpsc::Receiver<Instant>,
        std::thread::JoinHandle<()>,
    ) {
        let shared = Arc::new((
            Mutex::new(UiNativeDeadlineWatchState::default()),
            Condvar::new(),
        ));
        let (sender, posted) = mpsc::channel();
        let watched = Arc::clone(&shared);
        let thread = std::thread::spawn(move || {
            watch(&watched, |()| sender.send(Instant::now()).is_ok());
        });
        (shared, posted, thread)
    }

    fn watch_of(
        shared: &Arc<(Mutex<UiNativeDeadlineWatchState>, Condvar)>,
    ) -> super::UiNativeDeadlineWatch {
        super::UiNativeDeadlineWatch {
            shared: Arc::clone(shared),
            thread: None,
        }
    }

    #[test]
    fn a_deadline_the_loop_does_not_reach_posts_one_wake_after_it() {
        let (shared, posted, thread) = started();
        let armed = Instant::now() + Duration::from_millis(10);
        let watch = watch_of(&shared);
        watch.reached(Some(armed), true);
        let woke = posted.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(woke >= armed + WATCH_SLACK);
        // Window events arrive before the posted wake is dispatched. They
        // report the same overdue deadline and must not post it again.
        watch.reached(Some(armed), false);
        watch.reached(Some(armed), false);
        assert!(posted.recv_timeout(Duration::from_millis(200)).is_err());
        // Once the wake has progressed timed work, the next deadline posts.
        let next = Instant::now() + Duration::from_millis(10);
        watch.reached(Some(next), true);
        assert!(posted.recv_timeout(Duration::from_secs(5)).unwrap() >= next);
        drop(watch);
        thread.join().unwrap();
    }

    #[test]
    fn a_deadline_the_loop_reaches_or_withdraws_first_posts_nothing() {
        let (shared, posted, thread) = started();
        let watch = watch_of(&shared);
        watch.reached(Some(Instant::now() + Duration::from_millis(500)), true);
        // The loop progresses on its own and waits for nothing further.
        watch.reached(None, true);
        watch.reached(Some(Instant::now() + Duration::from_millis(500)), true);
        // A later deadline replaces the earlier one instead of joining it.
        watch.reached(Some(Instant::now() + Duration::from_secs(60)), true);
        assert!(posted.recv_timeout(Duration::from_millis(900)).is_err());
        drop(watch);
        thread.join().unwrap();
    }
}
