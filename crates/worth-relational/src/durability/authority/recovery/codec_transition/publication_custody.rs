//! Retain native codec phase custody through detached preparation and settlement.
use std::sync::Arc;

/// The memory obligation makes this an owning runtime lifecycle value. Sharing
/// this allocation carries one exact transition occurrence, never caller text.
#[derive(Debug)]
pub(crate) struct RecoveredCodecPublicationCustody {
    _memory: worth_execution::ExecutionMemoryReservation,
}
impl RecoveredCodecPublicationCustody {
    pub(super) fn new(memory: worth_execution::ExecutionMemoryReservation) -> Arc<Self> {
        Arc::new(Self { _memory: memory })
    }
}
impl PartialEq for RecoveredCodecPublicationCustody {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl Eq for RecoveredCodecPublicationCustody {}

/// Issued only after the native owner validates protected payload preservation
/// and prepares the complete target index image for this exact occurrence.
pub(crate) struct AdmittedRecoveredCodecPublication {
    custody: Arc<RecoveredCodecPublicationCustody>,
}
impl AdmittedRecoveredCodecPublication {
    pub(super) fn new(custody: Arc<RecoveredCodecPublicationCustody>) -> Self {
        Self { custody }
    }
    pub(crate) fn admits(&self, custody: &Arc<RecoveredCodecPublicationCustody>) -> bool {
        Arc::ptr_eq(&self.custody, custody)
    }
}
