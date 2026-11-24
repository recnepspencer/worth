//! Coordinate the existing Complete capture court with the retained owner cut.
use super::super::{capture_pause::CapturePause, owner::RelationalBranchRetentionOwnerInner};
use std::sync::{
    mpsc::{Receiver, SyncSender},
    Arc, Mutex,
};

#[doc(hidden)]
pub struct ComponentCaptureForTest {
    owner: Arc<RelationalBranchRetentionOwnerInner>,
}

impl ComponentCaptureForTest {
    pub fn external_cut_held_for_test(&self) -> bool {
        self.owner.external_cut.try_lock().is_err()
    }
}

impl Drop for ComponentCaptureForTest {
    fn drop(&mut self) {
        let slot = self
            .owner
            .component_capture_pause
            .get()
            .expect("armed component capture");
        *slot.lock().unwrap_or_else(|error| error.into_inner()) = None;
    }
}

#[doc(hidden)]
pub fn arm_component_capture_for_test(
    history: &crate::runtime::state::subsystems::history::HistorySubsystem,
    entered: SyncSender<()>,
    resume: Receiver<()>,
) -> ComponentCaptureForTest {
    let owner = history
        .retention_binding()
        .owner
        .upgrade()
        .expect("retention owner");
    let slot = owner
        .component_capture_pause
        .get_or_init(|| Mutex::new(None));
    let mut pause = slot.lock().unwrap_or_else(|error| error.into_inner());
    assert!(pause.is_none(), "component capture control already armed");
    *pause = Some(Arc::new(CapturePause {
        entered,
        resume: Mutex::new(resume),
    }));
    drop(pause);
    ComponentCaptureForTest { owner }
}
