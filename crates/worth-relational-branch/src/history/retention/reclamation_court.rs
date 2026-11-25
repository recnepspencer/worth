//! Receiver-local checkpoints for the existing retired root reclamation courts.
use super::{capture_pause::CapturePause, owner, RelationalBranchRetentionOwner};
use std::sync::{
    mpsc::{Receiver, SyncSender},
    Mutex,
};

#[derive(Debug, Default)]
pub(super) struct ReclamationCourt {
    drop_pause: Mutex<Option<CapturePause>>,
    contention: Mutex<Option<SyncSender<()>>>,
}

impl ReclamationCourt {
    pub(super) fn pause_before_drop(&self) {
        let pause = self
            .drop_pause
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if let Some(pause) = pause {
            pause.wait();
        }
    }

    pub(super) fn signal_contention(&self) {
        let signal = self
            .contention
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if let Some(signal) = signal {
            let _ = signal.send(());
        }
    }
}

#[doc(hidden)]
pub struct ReclamationPauseForTest<'owner> {
    owner: &'owner RelationalBranchRetentionOwner,
    court: &'owner ReclamationCourt,
}

impl ReclamationPauseForTest<'_> {
    pub fn observe_contention_for_test(&self, signal: SyncSender<()>) {
        let mut contention = self
            .court
            .contention
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        assert!(contention.is_none(), "reclamation contention already armed");
        *contention = Some(signal);
    }

    pub fn reclamation_gate_held_for_test(&self) -> bool {
        self.owner.inner.reclamation_gate.try_lock().is_err()
    }
}

impl Drop for ReclamationPauseForTest<'_> {
    fn drop(&mut self) {
        self.court
            .drop_pause
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        self.court
            .contention
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
    }
}

impl RelationalBranchRetentionOwner {
    #[doc(hidden)]
    pub fn pause_reclamation_drop_for_test(
        &self,
        entered: SyncSender<()>,
        resume: Receiver<()>,
    ) -> ReclamationPauseForTest<'_> {
        let court = self
            .inner
            .reclamation_court
            .get_or_init(ReclamationCourt::default);
        let mut pause = court
            .drop_pause
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        assert!(pause.is_none(), "reclamation drop already armed");
        *pause = Some(CapturePause {
            entered,
            resume: Mutex::new(resume),
        });
        ReclamationPauseForTest { owner: self, court }
    }

    #[doc(hidden)]
    pub fn external_cut_held_for_test(&self) -> bool {
        self.inner.external_cut.try_lock().is_err()
    }

    #[doc(hidden)]
    pub fn retired_root_unique_bytes_for_test(&self) -> Vec<u64> {
        self.inner
            .read_retired_roots()
            .values()
            .map(|entry| {
                entry
                    .root
                    .reclaimable_unique_authoritative_bytes()
                    .expect("fixture root byte count")
            })
            .collect()
    }

    #[doc(hidden)]
    pub fn set_reclaimed_unique_bytes_for_test(&self, bytes: u64) {
        owner::lock_maintenance_counters(&self.inner).reclamation_unique_authoritative_bytes =
            bytes;
    }
}
