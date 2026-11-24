//! Actual prerequisite publication custody, distinct from owned output members.
use crate::mvcc::publication::application_publication::{
    RelationalRequiredOutputPublication, RequiredOutputPublicationMeaning,
};
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use std::sync::Arc;

/// Request-local carriage of an existing Native publication or an admitted part.
/// Descriptive routes alone cannot construct this input or enroll a writer.
#[derive(Clone, Debug)]
pub struct RelationalRequiredOutputInput {
    pub(super) proposal: RelationalMutationProposalIdentity,
    pub(super) issuance: Arc<InputIssuance>,
    pub(super) origin: InputOrigin,
}

#[derive(Debug)]
pub(super) struct InputIssuance {
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Debug, Default)]
pub(crate) struct RequiredOutputInputs {
    pub(crate) meanings: Vec<RequiredOutputPublicationMeaning>,
    pub(crate) memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Clone, Debug)]
pub(super) enum InputOrigin {
    Current(RelationalRequiredOutputPublication),
    Applied(usize),
    Restored {
        index: usize,
        source: crate::identity::data::EntityId,
    },
}
