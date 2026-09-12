//! Certification scheduling at the actual admitted read-call boundary.
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};

use super::RecordReadSession;
use crate::physical_runtime::PhysicalWorkExecution;

pub struct CertificationPhysicalReadCallPauseGate {
    arrival: mpsc::Receiver<()>,
    release: Option<mpsc::SyncSender<()>>,
    execution: PhysicalWorkExecution,
}

pub(super) struct PhysicalReadCallPause {
    arrival: mpsc::SyncSender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}

impl RecordReadSession {
    /// Pauses a real chunk/copy call after it owns its runtime reference.
    /// Dropping the returned gate always releases the paused call.
    pub fn certification_pause_next_read_call(&mut self) -> CertificationPhysicalReadCallPauseGate {
        let (arrived, arrival) = mpsc::sync_channel(1);
        let (release, released) = mpsc::sync_channel(1);
        self.read_pause = Some(PhysicalReadCallPause {
            arrival: arrived,
            release: Mutex::new(released),
        });
        CertificationPhysicalReadCallPauseGate {
            arrival,
            release: Some(release),
            execution: self.execution.clone(),
        }
    }
}

impl PhysicalReadCallPause {
    pub(super) fn arrive_and_wait(&self) {
        let _ = self.arrival.try_send(());
        let _ = self
            .release
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .recv();
    }
}

impl CertificationPhysicalReadCallPauseGate {
    pub fn await_arrival(&self) -> bool {
        self.arrival.recv_timeout(Duration::from_secs(5)).is_ok()
    }

    /// Observes the real shutdown drain blocked by the admitted read call.
    pub fn await_shutdown_drain(&self) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.execution.read_call_drain_is_waiting() {
                return true;
            }
            std::thread::yield_now();
        }
        false
    }

    pub fn release(mut self) {
        self.release.take();
    }
}
