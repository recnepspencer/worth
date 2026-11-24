//! Bind continuation custody to an admitted runtime candidate.
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::producer_posture::RelationalCoupledProducerPostureOwnerPort as _;
use super::{part, RelationalAuthoredCoupledCandidate};
use crate::execution::request_stage::checkpoint;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;

pub(crate) use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::continuation::{application, custody};
pub(super) use custody::ContinuationState;
pub use custody::RelationalCoupledProducerContinuation;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Whether part `index` can derive its chain's next round: exactly when
    /// `next_producer_round` would admit it.
    pub fn continues_owned_route(&self, index: usize) -> bool {
        super::route_selection::continuation_refusal(&self.parts, self.binding, index).is_none()
    }

    /// Begin another observation of a real owned route. This does not admit a
    /// second publication or weaken first-entry duplicate-route admission.
    pub fn next_producer_round(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
    ) -> Result<RelationalCoupledProducerContinuation, TransactionCommitError> {
        if let Some((cause, subject)) =
            super::route_selection::continuation_refusal(&self.parts, self.binding, index)
        {
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
