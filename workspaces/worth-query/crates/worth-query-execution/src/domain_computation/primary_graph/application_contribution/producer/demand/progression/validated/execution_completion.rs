//! Finish the prepaid row publication on the canonical application owner.
use super::execution_preparation::CompletionRefusal;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::execution;
use crate::domain_computation::primary_graph::application_output_demand::RunningSelectedExecution;

pub(super) fn finish(
    result: Result<
        execution::outcome::PreparedProducerExecutionOutcome,
        execution::ProducerExecutionStop,
    >,
    selected_execution_finish: RunningSelectedExecution<'_>,
    retained_resources: Option<crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerDemandResources>,
    _request_admission: &mut InvalidationEditAdmission,
) -> Result<CheckpointProgress, CompletionRefusal> {
    let mut receipt = match result.map(|prepared| prepared.into_parts()) {
        Ok((execution::ProducerExecutionOutcome::Committed(receipt), ready_backing)) => {
            (receipt, ready_backing)
        }
        Ok((execution::ProducerExecutionOutcome::Stable(stable), ready_backing)) => {
            let completion = ready_backing.complete(
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryCompletedOutputDemand {
                        authority: crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Stable(stable),
                        readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
                        resources: retained_resources,
                    },
                );
            let checkpoint = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint::Ready(completion);
            if let Err((denial, checkpoint)) = selected_execution_finish.publish(checkpoint) {
                return Err(CompletionRefusal::unpublished(denial, checkpoint));
            }
            return Ok(CheckpointProgress::Advanced);
        }
        Err(execution::ProducerExecutionStop::RequestAdmissionDenied(denial)) => {
            selected_execution_finish.relinquish();
            return Err(CompletionRefusal::denied(denial.into_denial()));
        }
        Err(execution::ProducerExecutionStop::LiveOutputNotReused { producer, reason }) => {
            selected_execution_finish.relinquish();
            return Err(CompletionRefusal::denied(
                execution::ProducerExecutionStop::live_output_not_reused(producer, reason),
            ));
        }
        Err(execution::ProducerExecutionStop::ExecutionStopped(mut denial)) => {
            selected_execution_finish.failure(&mut denial);
            return Err(CompletionRefusal::denied(denial));
        }
    };
    let publication = OwnPublication(receipt.0.committed_product_publication().clone());
    #[cfg(feature = "test-query-execution-observer")]
    super::super::caller_pass_observation::exhaust_after_commit(_request_admission);
    let delivery = receipt.0
            .take_performed_relational_product_change()
            .map_or(
                crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery::NoChange,
                crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery::Change,
            );
    let checkpoint = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint::Published {
            receipt: receipt.0,
            delivery,
            ready_backing: receipt.1,
        };
    if let Err((denial, checkpoint)) = selected_execution_finish.publish(checkpoint) {
        return Err(CompletionRefusal::unpublished(denial, checkpoint));
    }
    Ok(CheckpointProgress::Published(publication))
}
