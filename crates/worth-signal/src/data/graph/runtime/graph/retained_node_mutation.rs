//! Immediate node-owned writes using the selected evaluation storage posture.
use super::retained_node_edit::RetainedNodeEditDenial;
use super::retained_node_edit::{RetainedNodeEditOutcome, RetainedNodeEditPreparation};
use super::SignalGraph;
use crate::data::error::SignalError;
use crate::data::graph::storage::NodeEvaluationMutation;
use crate::data::handle::NodeId;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
};
use crate::data::retained_storage::{
    RetainedStoragePreparationDenial, SignalConditionalRetentionDenial,
};
use crate::logic::evaluation::EvaluationWork;

impl SignalGraph {
    /// The caller supplies a bound for new callback allocations; the arena
    /// admits existing payload copies and all staged backing before allocation.
    /// Conditional callers lend the attempt allowance, never replenish it.
    pub(in crate::data::graph) fn mutate_evaluation_node<R>(
        &mut self,
        node: NodeId,
        growth: Charge,
        work: &mut EvaluationWork<'_>,
        edit: impl FnOnce(&mut NodeEvaluationMutation<'_>) -> R,
    ) -> Result<R, SignalError> {
        self.validate_handle(node)?;
        if self.arena.retained_node_ledger.is_none() {
            return Ok(edit(&mut self.node_evaluation_mutation(node)?));
        }
        match work {
            EvaluationWork::Conditional(work) => {
                self.mutate_reserved_evaluation_node(node, growth, work, edit)
            }
            EvaluationWork::Ordinary => {
                let maximum = self
                    .installed_runtime_policy()
                    .conditional_evaluation_budget()
                    .maximum_attempt_visits;
                self.mutate_reserved_evaluation_node(node, growth, &mut Work::new(maximum), edit)
            }
        }
    }

    fn mutate_reserved_evaluation_node<R>(
        &mut self,
        node: NodeId,
        growth: Charge,
        work: &mut Work,
        edit: impl FnOnce(&mut NodeEvaluationMutation<'_>) -> R,
    ) -> Result<R, SignalError> {
        let ledger = self
            .arena
            .retained_node_ledger
            .as_ref()
            .expect("selected storage ledger");
        let _growth = ledger.reserve(0, growth).map_err(map_retention)?;
        let maximum = self
            .installed_runtime_policy()
            .conditional_evaluation_budget()
            .maximum_retained_bytes;
        let maximum = usize::try_from(maximum)
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
        let prepared = self
            .arena
            .prepare_retained_node_edits(
                ledger,
                &[node.index() as usize],
                Charge::capacity::<u8>(maximum).map_err(map_accounting)?,
                work,
                |payloads| {
                    let payload = &mut payloads[0];
                    edit(&mut NodeEvaluationMutation::draft(
                        &mut payload.hot,
                        &mut payload.warm,
                        &mut payload.cold,
                    ))
                },
            )
            .map_err(map_edit)?;
        let prepared = match prepared {
            RetainedNodeEditPreparation::Ready(prepared) => prepared,
            RetainedNodeEditPreparation::Rejected { denial, .. } => return Err(map_edit(denial)),
        };
        match prepared.install(&mut self.arena) {
            RetainedNodeEditOutcome::Installed { output, .. } => Ok(output),
            RetainedNodeEditOutcome::Rejected { denial, .. } => Err(map_edit(denial)),
        }
    }
}

pub(in crate::data::graph) fn map_accounting(
    denial: RetainedStoragePreparationDenial,
) -> SignalError {
    match denial {
        RetainedStoragePreparationDenial::WorkExhausted { maximum_visits } => {
            SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }
        }
        _ => SignalError::EvaluationStorageCapacityExhausted,
    }
}
pub(in crate::data::graph) fn map_retention(
    denial: SignalConditionalRetentionDenial,
) -> SignalError {
    match denial {
        SignalConditionalRetentionDenial::CapacityExhausted => {
            SignalError::EvaluationStorageCapacityExhausted
        }
        _ => SignalError::EvaluationStorageUnavailable,
    }
}
pub(in crate::data::graph) fn map_edit(denial: RetainedNodeEditDenial) -> SignalError {
    match denial {
        RetainedNodeEditDenial::Accounting(denial) => map_accounting(denial),
        RetainedNodeEditDenial::Retention(denial) => map_retention(denial),
        RetainedNodeEditDenial::CapacityExhausted { .. } => {
            SignalError::EvaluationStorageCapacityExhausted
        }
        _ => SignalError::EvaluationStorageUnavailable,
    }
}
