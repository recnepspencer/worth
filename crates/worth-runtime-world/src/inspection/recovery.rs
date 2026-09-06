use crate::identity::RuntimeWorldOwnerIdentity;
use crate::lifecycle::RuntimeWorldInstant;
use crate::recovery::ProductUnpublishedRecoveryHandle;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldRecoveryRecordState {
    Active,
    Abandoned,
    Retained,
    Busy,
}
/// Descriptive page row. A busy update deliberately exposes no stale age.
#[derive(Debug)]
pub struct RuntimeWorldRecoveryRow {
    pub(crate) handle: ProductUnpublishedRecoveryHandle,
    pub(crate) state: RuntimeWorldRecoveryRecordState,
    pub(crate) admitted_at: Option<RuntimeWorldInstant>,
    pub(crate) age_ticks: Option<u64>,
    pub(crate) deadline_expired: Option<bool>,
}
impl RuntimeWorldRecoveryRow {
    pub fn handle(&self) -> &ProductUnpublishedRecoveryHandle {
        &self.handle
    }
    pub fn state(&self) -> RuntimeWorldRecoveryRecordState {
        self.state
    }
    pub fn admitted_at(&self) -> Option<RuntimeWorldInstant> {
        self.admitted_at
    }
    pub fn age_ticks(&self) -> Option<u64> {
        self.age_ticks
    }
    pub fn deadline_expired(&self) -> Option<bool> {
        self.deadline_expired
    }
}
/// Descriptive position in one live catalog. It carries no recovery authority.
/// Removing or reusing an earlier slot does not invalidate the continuation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWorldRecoveryCursor {
    pub(crate) owner: RuntimeWorldOwnerIdentity,
    pub(crate) catalog_affinity: usize,
    pub(crate) next_slot: usize,
}

/// Every examined slot, including a vacancy, spends one unit of the caller
/// bound. Pages follow slot order and observe a live, unfrozen catalog.
#[derive(Debug)]
pub struct RuntimeWorldRecoveryPage {
    pub(crate) rows: Vec<RuntimeWorldRecoveryRow>,
    pub(crate) next_after: Option<RuntimeWorldRecoveryCursor>,
    pub(crate) examined: usize,
    pub(crate) observed_at: RuntimeWorldInstant,
}
impl RuntimeWorldRecoveryPage {
    pub fn rows(&self) -> &[RuntimeWorldRecoveryRow] {
        &self.rows
    }
    pub fn examined(&self) -> usize {
        self.examined
    }
    pub fn next_after(&self) -> Option<&RuntimeWorldRecoveryCursor> {
        self.next_after.as_ref()
    }
    pub fn observed_at(&self) -> RuntimeWorldInstant {
        self.observed_at
    }
}
