//! Linear preparation custody over an actual owned part, never a write grant.
use super::super::super::custody;
use super::super::refusals;
use contracts::identity::data::record_ids::EntityId;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::{
    RelationalMutationProposalIdentity, RelationalMutationProposalIdentityOwnerPort as _,
};
/// Exact candidate/round custody. A new execution still needs its own current
/// operation admission, and Native still judges its entire body before adoption.
pub struct RelationalCoupledProducerContinuation {
    identity: RelationalMutationProposalIdentity,
    sequence: u64,
    part: usize,
    phase: usize,
    source: EntityId,
}
#[derive(Default)]
pub struct ContinuationState {
    sequence: u64,
    pending: Option<u64>,
}
impl ContinuationState {
    pub fn issue(
        &mut self,
        identity: &RelationalMutationProposalIdentity,
        part: usize,
        phase: usize,
        source: EntityId,
    ) -> Result<RelationalCoupledProducerContinuation, TransactionCommitError> {
        self.sequence = self.sequence.checked_add(1).ok_or_else(|| {
            custody::execution_stop(
                contracts::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
            )
        })?;
        self.pending = Some(self.sequence);
        Ok(RelationalCoupledProducerContinuation {
            identity: identity.clone(),
            sequence: self.sequence,
            part,
            phase,
            source,
        })
    }
    pub fn consume(
        &mut self,
        token: RelationalCoupledProducerContinuation,
        identity: &RelationalMutationProposalIdentity,
        phase: usize,
    ) -> Result<usize, TransactionCommitError> {
        if !identity.same_proposal_sequence(&token.identity) {
            return Err(refusals::reject(
                Cause::ForeignCoupledContinuation,
                token.source,
            ));
        }
        if self.pending != Some(token.sequence) || token.phase != phase {
            return Err(refusals::reject(
                Cause::StaleCoupledContinuation,
                token.source,
            ));
        }
        self.pending = None;
        Ok(token.part)
    }
}

impl RelationalCoupledProducerContinuation {
    /// Only the closed corruption receiver can replay an actually issued phase.
    pub(super) fn duplicate_for_replay_probe(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            sequence: self.sequence,
            part: self.part,
            phase: self.phase,
            source: self.source,
        }
    }
}

#[doc(hidden)]
pub trait RelationalCoupledProducerContinuationOwnerPort: Sized {
    fn duplicate_for_replay_probe(&self) -> Self;
}

impl RelationalCoupledProducerContinuationOwnerPort for RelationalCoupledProducerContinuation {
    fn duplicate_for_replay_probe(&self) -> Self {
        self.duplicate_for_replay_probe()
    }
}
