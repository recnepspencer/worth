//! Linear preparation custody over an actual owned part, never a write grant.
use super::{part, RelationalAuthoredCoupledCandidate};
use crate::execution::request_stage::checkpoint;
use crate::identity::data::EntityId;
use crate::mvcc::RelationalMutationProposalIdentity;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::transactions::data::TransactionCommitError;
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
    /// Begin another observation of a real owned route. This does not admit a
    /// second publication or weaken first-entry duplicate-route admission.
    pub fn next_producer_round(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
    ) -> Result<RelationalCoupledProducerContinuation, TransactionCommitError> {
        let Some(part) = self.parts.get(index) else {
            return Err(part::reject(
                Cause::MissingCoupledOwner,
                self.authored_source(),
            ));
        };
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
        // The route map's Native half: only an Edit may complete a route by
        // verification, so only an Edit's owned route may observe it again.
        if !self.permits_unchanged_output_selection() {
            return Err(part::reject(
                Cause::CoupledContinuationRequiresEditSource,
                meaning.source_scope,
            ));
        }
        if part.publication.owned().is_none()
            || self
                .binding
                .installed
                .source_scope_ownership
                .scope_kind(producer)
                .is_none()
        {
            return Err(part::reject(
                Cause::CoupledContinuationRequiresOwnedRoute,
                meaning.source_scope,
            ));
        }
        self.continuation
            .issue(self.identity, index, self.parts.len(), meaning.source_scope)
    }
}

mod corruption;
mod round;
