//! Closed estate sources retain their exact declared result family.
use super::closed_source_evidence::{ClosedSourceEvidence, SourceResult};
use bank_domain::schema::{DisburseEstate, NotifyEstateDeath};
use worth_query_host::facade::application_entry::WorthQueryClosedOperation;
pub(super) enum ClosedMutationEstate {
    Notification(ClosedSourceEvidence<SourceResult<NotifyEstateDeath>>),
    Disbursement(ClosedSourceEvidence<SourceResult<DisburseEstate>>),
}
impl ClosedMutationEstate {
    pub(super) fn closed(&self) -> &WorthQueryClosedOperation {
        match self {
            Self::Notification(owner) => owner.closed(),
            Self::Disbursement(owner) => owner.closed(),
        }
    }
    pub(super) fn closed_mut(&mut self) -> &mut WorthQueryClosedOperation {
        match self {
            Self::Notification(owner) => owner.closed_mut(),
            Self::Disbursement(owner) => owner.closed_mut(),
        }
    }
}
