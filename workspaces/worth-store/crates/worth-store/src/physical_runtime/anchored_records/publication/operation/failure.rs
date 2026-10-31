use super::*;

#[derive(Debug)]
pub enum AnchoredRecordOperationFailure {
    Publication(AnchoredRecordPublicationFailure),
    Busy,
    ForeignReceiver,
    PublicationOwnerReleased,
    Stopped,
    OriginalRequestStopped(worth_execution::MapKernelStop),
    Panicked,
    StepCustody(AnchoredRecordStepCustodyRefusal),
    StepSettlement(AnchoredRecordStepSettlementFailure),
}
impl From<AnchoredRecordPublicationFailure> for AnchoredRecordOperationFailure {
    fn from(failure: AnchoredRecordPublicationFailure) -> Self {
        Self::Publication(failure)
    }
}

/// Owns stopped custody and its actual retained prefix/terminal facts.
#[must_use]
pub struct AnchoredRecordPublicationFailureCustody {
    pub(super) stopped: AnchoredRecordStoppedPublication,
    pub(super) cause: AnchoredRecordOperationFailure,
}
impl AnchoredRecordPublicationFailureCustody {
    pub(in crate::physical_runtime::anchored_records) fn into_publication_failure(
        self,
    ) -> AnchoredRecordPublicationFailure {
        match self {
            Self {
                stopped: _,
                cause: AnchoredRecordOperationFailure::Publication(failure),
            } => failure,
            other => AnchoredRecordPublicationFailure::OwnedPublication(Box::new(other)),
        }
    }
    pub fn cause(&self) -> &AnchoredRecordOperationFailure {
        &self.cause
    }
    pub fn observe(&self) -> AnchoredRecordPublicationObservation {
        self.stopped.observe()
    }
    pub fn into_stopped(self) -> AnchoredRecordStoppedPublication {
        self.stopped
    }
}
