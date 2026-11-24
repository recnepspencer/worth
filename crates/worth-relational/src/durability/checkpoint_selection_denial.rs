//! Refusal to select a checkpoint at the canonical publication cut.
pub(crate) enum CanonicalCheckpointAdmissionError {
    PublicationInFlight,
    PerformedPublicationRequiresSettlement(crate::history::data::CommitId),
}
