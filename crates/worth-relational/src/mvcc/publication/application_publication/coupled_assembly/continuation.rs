//! Linear preparation custody over an actual owned part, never a write grant.
use super::{part, RelationalAuthoredCoupledCandidate};
use crate::execution::request_stage::checkpoint;
use crate::identity::data::EntityId;
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;
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
pub(super) struct ContinuationState {
    sequence: u64,
    pending: Option<u64>,
}
impl ContinuationState {
    fn issue(
        &mut self,
        identity: &RelationalMutationProposalIdentity,
        part: usize,
        phase: usize,
        source: EntityId,
    ) -> Result<RelationalCoupledProducerContinuation, TransactionCommitError> {
        self.sequence = self.sequence.checked_add(1).ok_or_else(|| {
            super::custody::execution_stop(
                crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
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
    pub(super) fn consume(
        &mut self,
        token: RelationalCoupledProducerContinuation,
        identity: &RelationalMutationProposalIdentity,
        phase: usize,
    ) -> Result<usize, TransactionCommitError> {
        if !identity.same_proposal_sequence(&token.identity) {
            return Err(part::reject(
                Cause::ForeignCoupledContinuation,
                token.source,
            ));
        }
        if self.pending != Some(token.sequence) || token.phase != phase {
            return Err(part::reject(Cause::StaleCoupledContinuation, token.source));
        }
        self.pending = None;
        Ok(token.part)
    }
}
impl RelationalAuthoredCoupledCandidate<'_> {
    /// Why part `index` cannot derive its chain's next round, if it cannot:
    /// the complete continuation admission, with the record each refusal
    /// names. `next_producer_round` refuses with exactly this answer, so a
    /// caller that reopens appended routes (`continues_owned_route`) asks the
    /// same question Native answers when the round arrives.
    fn continuation_refusal(&self, index: usize) -> Option<(Cause, EntityId)> {
        let Some(part) = self.parts.get(index) else {
            return Some((Cause::MissingCoupledOwner, self.authored_source()));
        };
        let source = part
            .publication
            .producing_meaning()
            .expect("admitted part")
            .source_scope;
        // Only an Edit may observe its already appended owned route again;
        // a retirement or restoration still requires its exact preserving
        // participants.
        if !self.permits_unchanged_output_selection() {
            return Some((Cause::CoupledContinuationRequiresEditSource, source));
        }
        // Only an owned publication of a producer that owns a source scope.
        let producer = part
            .publication
            .lineage_attribution()
            .expect("actual producer")
            .0;
        if part.publication.owned().is_none()
            || self
                .binding
                .installed
                .source_scope_ownership
                .scope_kind(producer)
                .is_none()
        {
            return Some((Cause::CoupledContinuationRequiresOwnedRoute, source));
        }
        None
    }

    /// Whether part `index` can derive its chain's next round: exactly when
    /// `next_producer_round` would admit it.
    pub fn continues_owned_route(&self, index: usize) -> bool {
        self.continuation_refusal(index).is_none()
    }

    /// Begin another observation of a real owned route. This does not admit a
    /// second publication or weaken first-entry duplicate-route admission.
    pub fn next_producer_round(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
    ) -> Result<RelationalCoupledProducerContinuation, TransactionCommitError> {
        if let Some((cause, subject)) = self.continuation_refusal(index) {
            return Err(part::reject(cause, subject));
        }
        let part = &self.parts[index];
        let meaning = part.publication.producing_meaning().expect("admitted part");
        let producer = part
            .publication
            .lineage_attribution()
            .expect("actual producer")
            .0;
        execution
            .run_native_with_checkpoint(|context| {
                checkpoint(context, (meaning.family.len() + producer.len() + 1) as u64)
                    .map_err(super::custody::execution_stop)
            })
            .map_err(super::custody::execution_stop)?
            .0?;
        let source = meaning.source_scope;
        self.continuation
            .issue(self.identity, index, self.applied_parts(), source)
    }
}

mod corruption;
mod round;
pub(super) use super::refusals::postponed;
