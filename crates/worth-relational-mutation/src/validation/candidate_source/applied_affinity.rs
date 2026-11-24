//! The exact proposal and registration attached to an applied Source projection.
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;
use worth_relational_contracts::internal_port::validation::candidate_source::program::ManagedIndexSourceProgram;

pub struct AppliedSourceAffinity {
    pub proposal: RelationalMutationProposalIdentity,
    pub program: ManagedIndexSourceProgram,
    pub scope: EntityId,
    pub generation: u64,
    pub _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
