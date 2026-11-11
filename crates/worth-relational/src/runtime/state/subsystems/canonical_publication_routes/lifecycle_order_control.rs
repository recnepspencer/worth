//! Owner-scoped rendezvous for publication/exclusive-snapshot lock-order races.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::Duration;

#[derive(Default)]
pub(super) struct LifecycleOrderControl {
    exclusive: Mutex<Option<ExclusivePause>>,
    transition_contacts: AtomicU64,
}

struct ExclusivePause {
    reached: mpsc::SyncSender<()>,
    resume: mpsc::Receiver<()>,
}

impl LifecycleOrderControl {
    pub(super) fn transition_contact(&self) {
        self.transition_contacts.fetch_add(1, Ordering::Release);
    }
    pub(super) fn pause_exclusive(&self) {
        let pause = self.exclusive.lock().unwrap().take();
        if let Some(pause) = pause {
            pause.reached.send(()).unwrap();
            pause
                .resume
                .recv_timeout(Duration::from_secs(10))
                .expect("exclusive snapshot resumes");
        }
    }
}
