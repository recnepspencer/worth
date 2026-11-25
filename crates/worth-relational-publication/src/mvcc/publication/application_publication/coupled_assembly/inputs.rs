//! Actual prerequisite publication custody, distinct from owned output members.
use crate::mvcc::publication::application_publication::RelationalRequiredOutputPublication;
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
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
pub struct InputIssuance {
    pub _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Debug, Default)]
pub struct RequiredOutputInputs {
    pub meanings: Vec<RequiredOutputPublicationMeaning>,
    pub(crate) memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Clone, Debug)]
pub enum InputOrigin {
    Current(RelationalRequiredOutputPublication),
    Applied(usize),
    Restored {
        index: usize,
        source: worth_relational_contracts::internal_port::identity::data::record_ids::EntityId,
    },
}

#[doc(hidden)]
pub trait RelationalRequiredOutputInputOwnerPort: Sized {
    fn from_issued_input(
        proposal: RelationalMutationProposalIdentity,
        issuance: Arc<InputIssuance>,
        origin: InputOrigin,
    ) -> Self;
}

impl RelationalRequiredOutputInputOwnerPort for RelationalRequiredOutputInput {
    fn from_issued_input(
        proposal: RelationalMutationProposalIdentity,
        issuance: Arc<InputIssuance>,
        origin: InputOrigin,
    ) -> Self {
        Self {
            proposal,
            issuance,
            origin,
        }
    }
}
