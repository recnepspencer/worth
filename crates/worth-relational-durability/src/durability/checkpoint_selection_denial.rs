//! Refusal to select a checkpoint at the canonical publication cut.
pub enum CanonicalCheckpointAdmissionError {
    PublicationInFlight,
    PerformedPublicationRequiresSettlement(
        worth_relational_contracts::internal_port::history::data::coordinates::CommitId,
    ),
}
