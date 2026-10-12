//! Receiver-owned, read-only names for shipped read consult regions.
use std::cell::Cell;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
thread_local! { static CURRENT: Cell<(usize, u8)> = const { Cell::new((0,0)) }; }
#[derive(Default)]
pub(super) struct NamedReadProbe {
    armed: AtomicBool,
}
#[derive(Clone, Copy)]
pub(crate) enum NamedRead {
    SourceProgramResolution = 2,
    GeneratedRecoverySourceSelection = 3,
}
pub struct WorthQueryNamedReadObservationForTest {
    probe: Arc<NamedReadProbe>,
    previous: bool,
}
impl Drop for WorthQueryNamedReadObservationForTest {
    fn drop(&mut self) {
        self.probe.armed.store(self.previous, Ordering::Release);
    }
}
pub(crate) struct NamedReadRegion {
    _probe: Option<Arc<NamedReadProbe>>,
    previous: Option<(usize, u8)>,
}
impl Drop for NamedReadRegion {
    fn drop(&mut self) {
        if let Some(previous) = self.previous {
            CURRENT.set(previous);
        }
    }
}
impl super::WorthQueryProductRuntime {
    pub fn observe_named_read_consults_for_test(&self) -> WorthQueryNamedReadObservationForTest {
        let previous = self.named_read_probe.armed.swap(true, Ordering::AcqRel);
        WorthQueryNamedReadObservationForTest {
            probe: Arc::clone(&self.named_read_probe),
            previous,
        }
    }
    pub(crate) fn enter_named_read(&self, name: NamedRead) -> NamedReadRegion {
        let previous = self
            .named_read_probe
            .armed
            .load(Ordering::Acquire)
            .then(|| CURRENT.replace((Arc::as_ptr(&self.named_read_probe) as usize, name as u8)));
        NamedReadRegion {
            _probe: previous.map(|_| Arc::clone(&self.named_read_probe)),
            previous,
        }
    }
    fn named_read_active(&self, name: NamedRead) -> bool {
        self.named_read_probe.armed.load(Ordering::Acquire)
            && CURRENT.get() == (Arc::as_ptr(&self.named_read_probe) as usize, name as u8)
    }
    pub fn source_program_resolution_is_active_for_test(&self) -> bool {
        self.named_read_active(NamedRead::SourceProgramResolution)
    }
    pub fn generated_recovery_source_selection_is_active_for_test(&self) -> bool {
        self.named_read_active(NamedRead::GeneratedRecoverySourceSelection)
    }
}
