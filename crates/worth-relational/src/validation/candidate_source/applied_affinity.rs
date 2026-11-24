//! The exact proposal and registration attached to an applied Source projection.
use super::program::ManagedIndexSourceProgram;
use crate::identity::data::EntityId;
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;

pub(super) struct AppliedSourceAffinity {
    pub(super) proposal: RelationalMutationProposalIdentity,
    pub(super) program: ManagedIndexSourceProgram,
    pub(super) scope: EntityId,
    pub(super) generation: u64,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
