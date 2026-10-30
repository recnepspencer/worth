//! Owner-scoped rendezvous while a live transition holds its reservation.

use std::sync::{mpsc, Mutex};
use std::time::Duration;

#[derive(Default)]
pub(super) struct BranchTransitionPublicationControlForTest {
    pause: Mutex<Option<PublicationPause>>,
}

struct PublicationPause {
    reached: mpsc::SyncSender<()>,
    resume: mpsc::Receiver<()>,
}

pub(crate) struct BranchTransitionPublicationPauseForTest {
    reached: mpsc::Receiver<()>,
    resume: mpsc::SyncSender<()>,
}

impl BranchTransitionPublicationPauseForTest {
    pub(crate) fn await_publication(&self) {
        self.reached
            .recv_timeout(Duration::from_secs(30))
            .expect("transition reached its publication step");
    }

    pub(crate) fn release(&self) {
        let _ = self.resume.try_send(());
    }
}

impl Drop for BranchTransitionPublicationPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}

impl BranchTransitionPublicationControlForTest {
    pub(super) fn arm(&self) -> BranchTransitionPublicationPauseForTest {
        let (reached, observed) = mpsc::sync_channel(1);
        let (resume, released) = mpsc::sync_channel(1);
        assert!(self
            .pause
            .lock()
            .unwrap()
            .replace(PublicationPause {
                reached,
                resume: released,
            })
            .is_none());
        BranchTransitionPublicationPauseForTest {
            reached: observed,
            resume,
        }
    }

    pub(super) fn pause(&self) {
        let pause = self.pause.lock().unwrap().take();
        if let Some(pause) = pause {
            pause.reached.send(()).unwrap();
            pause
                .resume
                .recv_timeout(Duration::from_secs(30))
                .expect("transition publication resumes");
        }
    }
}
