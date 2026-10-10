//! Unresolved native outcomes retain their original source preparation.
use super::super::WorthQueryApplicationMutationOutcome;
use worth_query_execution::facade::application_installation::WorthQueryUnpublishedProgramOutputSource;
/// Non-actionable custody. The native outcome is not a no-effect result and
/// cannot be advanced through ordinary or unpublished recovery entrances.
pub struct WorthQueryBlockedProgramSource<Denial, Value, Payload> {
    pub(in crate::application_entry::mutation) _preparation:
        WorthQueryUnpublishedProgramOutputSource,
    pub(in crate::application_entry::mutation) payload: Payload,
    pub(in crate::application_entry::mutation) outcome:
        WorthQueryApplicationMutationOutcome<Denial, Value>,
}
impl<Denial, Value, Payload> WorthQueryBlockedProgramSource<Denial, Value, Payload> {
    pub fn payload(&self) -> &Payload {
        &self.payload
    }
    pub fn outcome(&self) -> &WorthQueryApplicationMutationOutcome<Denial, Value> {
        &self.outcome
    }
}
