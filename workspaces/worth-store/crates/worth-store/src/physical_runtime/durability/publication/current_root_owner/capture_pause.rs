//! Test scheduling at the real observation/registration linearization boundary.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex, TryLockError,
};
use std::time::Duration;

use super::PhysicalCurrentRootOwner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificationReadRootCaptureStage {
    BeforeRootLock,
    AfterObservationBeforeRegistration,
}

pub struct CertificationReadRootCapturePauseGate {
    arrival: mpsc::Receiver<()>,
    publication_wait: mpsc::Receiver<()>,
    release: Option<mpsc::SyncSender<()>>,
}

pub(super) struct ReadRootCapturePause {
    stage: CertificationReadRootCaptureStage,
    armed: AtomicBool,
    arrival: mpsc::SyncSender<()>,
    publication_wait: mpsc::SyncSender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn pause_next_capture_for_certification(
        &self,
        stage: CertificationReadRootCaptureStage,
    ) -> CertificationReadRootCapturePauseGate {
        let (arrived, arrival) = mpsc::sync_channel(1);
        let (waited, publication_wait) = mpsc::sync_channel(1);
        let (release, released) = mpsc::sync_channel(1);
        *self.capture_pause.lock().unwrap() = Some(Arc::new(ReadRootCapturePause {
            stage,
            armed: AtomicBool::new(true),
            arrival: arrived,
            publication_wait: waited,
            release: Mutex::new(released),
        }));
        CertificationReadRootCapturePauseGate {
            arrival,
            publication_wait,
            release: Some(release),
        }
    }

    pub(super) fn pause_capture_at(&self, stage: CertificationReadRootCaptureStage) {
        let pause = self.capture_pause.lock().unwrap().clone();
        if let Some(pause) = pause {
            if pause.stage == stage && pause.armed.swap(false, Ordering::AcqRel) {
                let _ = pause.arrival.try_send(());
                let _ = pause.release.lock().unwrap().recv();
            }
        }
    }

    pub(super) fn observe_publication_lock_wait(&self) {
        let pause = self.capture_pause.lock().unwrap().clone();
        if let Some(pause) = pause {
            // Observe the same mutex the real publication path next acquires.
            // A split snapshot/register implementation cannot produce this
            // positive wait observation while capture is paused at the seam.
            if matches!(self.state.try_lock(), Err(TryLockError::WouldBlock)) {
                let _ = pause.publication_wait.try_send(());
            }
        }
    }
}

impl CertificationReadRootCapturePauseGate {
    pub fn await_arrival(&self) -> bool {
        self.arrival.recv_timeout(Duration::from_secs(5)).is_ok()
    }

    pub fn await_publication_lock_wait(&self) -> bool {
        self.publication_wait
            .recv_timeout(Duration::from_secs(5))
            .is_ok()
    }

    /// Dropping this gate also releases the paused acquisition.
    pub fn release(mut self) {
        self.release.take();
    }
}
