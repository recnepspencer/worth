use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionResolver, SignalConditionalExecutionRequest,
};
use crate::data::error::SignalError;
use crate::data::graph::storage::evaluation_partition::SignalPartitionConditionalDenial;
use crate::data::graph::SignalGraph;
use crate::data::output::NodeEvaluationResult;
use crate::data::retained_storage::RetainedStoragePreparation;

use super::{
    SignalConditionalEvaluationAdmission, SignalConditionalEvaluationState,
    SignalConditionalExecutionSlot, SignalConditionalServiceCompletion,
    SignalConditionalServiceExecutionDenial,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::branch::owner_services) fn execute_conditional_against_graph(
    graph: &mut SignalGraph,
    evaluation: &SignalConditionalEvaluationAdmission,
    evaluation_state: &mut SignalConditionalEvaluationState,
    request: SignalConditionalExecutionRequest<'_>,
    condition: &mut impl InstalledSignalConditionResolver,
    comparator: &mut impl ComparatorPolicyResolver,
    compute: impl FnOnce() -> Result<NodeEvaluationResult, SignalError>,
) -> Result<SignalConditionalServiceCompletion, SignalConditionalServiceExecutionDenial> {
    use SignalConditionalServiceExecutionDenial as Denial;

    if evaluation_state.slot.is_none() {
        let maximum_visits = graph
            .installed_runtime_policy()
            .conditional_evaluation_budget()
            .maximum_attempt_visits;
        let partition = evaluation
            .retained_basis
            .new_evaluation_partition_with_reserved_slot(
                &mut RetainedStoragePreparation::new(maximum_visits),
                &mut evaluation_state.admission_custody,
            )
            .map_err(Denial::SlotAdmission)?;
        evaluation_state.slot = Some(SignalConditionalExecutionSlot { partition });
    }
    let execution = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        evaluation_state
            .slot
            .as_mut()
            .expect("slot was admitted above")
            .partition
            .execute_conditional(graph, request, condition, comparator, compute)
    }));
    match execution {
        Ok(Ok(completion)) => Ok(SignalConditionalServiceCompletion::from_partition(
            completion, evaluation,
        )),
        Ok(Err(denial)) => Err(map_partition_denial(denial)),
        Err(payload) => {
            let slot = &mut evaluation_state
                .slot
                .as_mut()
                .expect("unwound slot remains installed")
                .partition;
            drop(slot.take_conditional_unwind());
            std::panic::resume_unwind(payload)
        }
    }
}

fn map_partition_denial(
    denial: SignalPartitionConditionalDenial,
) -> SignalConditionalServiceExecutionDenial {
    use SignalConditionalServiceExecutionDenial as Service;
    match denial {
        SignalPartitionConditionalDenial::Activation(error)
        | SignalPartitionConditionalDenial::StorageAdmission(error) => {
            Service::SlotAdmission(error)
        }
        SignalPartitionConditionalDenial::ObservationAdmission(denial) => {
            Service::ObservationAdmission(denial)
        }
        SignalPartitionConditionalDenial::UnconsumedUnwind => Service::UnconsumedUnwind,
    }
}
