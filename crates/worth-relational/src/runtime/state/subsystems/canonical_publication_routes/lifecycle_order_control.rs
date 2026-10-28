//! Owner-scoped rendezvous for publication/exclusive-snapshot lock-order races.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};

#[derive(Default)]
pub(super) struct LifecycleOrderControl {
    exclusive: Mutex<Option<ExclusivePause>>,
    transition_contacts: AtomicU64,
}

struct ExclusivePause {
    reached: mpsc::SyncSender<()>,
    resume: mpsc::Receiver<()>,
}

pub(crate) struct LifecycleSnapshotPause {
    reached: mpsc::Receiver<()>,
    resume: mpsc::SyncSender<()>,
}

impl LifecycleSnapshotPause {
    pub(crate) fn await_admission(&self) {
        self.reached
            .recv_timeout(Duration::from_secs(10))
            .expect("exclusive snapshot acquired lifecycle");
    }
    pub(crate) fn release(&self) {
        let _ = self.resume.try_send(());
    }
}

impl Drop for LifecycleSnapshotPause {
    fn drop(&mut self) {
        self.release();
    }
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

impl super::RelationalCanonicalPublicationRoutes {
    pub(crate) fn pause_exclusive_snapshot_for_test(&self) -> LifecycleSnapshotPause {
        let (reached, observed) = mpsc::sync_channel(1);
        let (resume, released) = mpsc::sync_channel(1);
        assert!(self
            .lifecycle_order
            .exclusive
            .lock()
            .unwrap()
            .replace(ExclusivePause {
                reached,
                resume: released
            })
            .is_none());
        LifecycleSnapshotPause {
            reached: observed,
            resume,
        }
    }
    pub(crate) fn transition_lifecycle_contacts_for_test(&self) -> u64 {
        self.lifecycle_order
            .transition_contacts
            .load(Ordering::Acquire)
    }
    pub(crate) fn await_transition_lifecycle_contact_for_test(&self, prior: u64) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while self.transition_lifecycle_contacts_for_test() == prior {
            assert!(
                Instant::now() < deadline,
                "transition reached lifecycle admission"
            );
            std::thread::yield_now();
        }
    }
}
