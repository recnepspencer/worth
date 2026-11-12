//! Finish the prepaid row publication on the canonical application owner.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::execution;
use crate::domain_computation::primary_graph::application_output_demand::RunningSelectedExecution;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint;

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression)
struct CompletionRefusal
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) denial:
        WorthQueryOutputDemandDenial,
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) checkpoint:
        Option<WorthQueryOutputCheckpoint>,
}
impl CompletionRefusal {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn denied(
        denial: WorthQueryOutputDemandDenial,
    ) -> Self {
        Self {
            denial,
            checkpoint: None,
        }
    }
    pub(super) fn unpublished(
        denial: WorthQueryOutputDemandDenial,
        checkpoint: WorthQueryOutputCheckpoint,
    ) -> Self {
        Self {
            denial,
            checkpoint: Some(checkpoint),
        }
    }
}

pub(super) fn finish(
    result: Result<
        execution::outcome::PreparedProducerExecutionOutcome,
        execution::ProducerExecutionStop,
    >,
    selected_execution_finish: RunningSelectedExecution<'_>,
    performed: &std::cell::RefCell<PerformedMembers>,
    readiness: super::super::required_wave::performed::PublicationReadiness,
    runtime_identity: u64,
    root: worth_relational::facade::identity::EntityId,
    original_publication: Option<std::sync::Arc<
        crate::domain_computation::primary_graph::application_output_demand::OriginalProducingPublication,
    >>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<CheckpointProgress, CompletionRefusal> {
    let receipt = match result.map(|prepared| prepared.into_parts()) {
        Ok((execution::ProducerExecutionOutcome::Unpublished(mut owner), ready_backing)) => {
            owner.retain_original_publication(original_publication);
            let checkpoint = WorthQueryOutputCheckpoint::PublicationPending {
                owner,
                ready_backing,
            };
            if let Err((denial, checkpoint)) = selected_execution_finish.publish(checkpoint) {
                return Err(CompletionRefusal::unpublished(denial, checkpoint));
            }
            return Ok(CheckpointProgress::Advanced);
        }
        Ok((execution::ProducerExecutionOutcome::Committed(receipt), ready_backing)) => {
            let mut record = performed.borrow_mut();
            let (receipt, recorded) = record
                .publication(readiness, receipt, |member| member.performed())
                .into_parts();
            super::super::caller_pass_observation::observe_publication(
                true,
                *recorded,
                runtime_identity,
                root,
            );
            (receipt.into_receipt(), ready_backing)
        }
        Ok((
            execution::ProducerExecutionOutcome::OwnEffectContinuation(receipt),
            ready_backing,
        )) => {
            // The original coupled publication already happened. Preserve its
            // existing Published delivery protocol without marking another
            // publication or issuing Stable/Current authority.
            (receipt, ready_backing)
        }
        Ok((execution::ProducerExecutionOutcome::Stable(stable), ready_backing)) => {
            let completion = ready_backing.complete(
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryCompletedOutputDemand {
                        authority: crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Stable(stable),
                        readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
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
    let (receipt, ready_backing) = receipt;
    let retained = crate::domain_computation::primary_graph::application_output_demand::OriginalProducingPublication::for_successor(
        execution, original_publication, (&receipt).into(),
    );
    let (original_publication, continuation_denial) = match retained {
        Ok(original) => (original, None),
        Err(denial) => (None, Some(denial)),
    };
    let mut receipt = (
        receipt,
        ready_backing.bind_original_producing_publication(original_publication),
    );
    own_effect_observation::record(Stage::Published);
    let publication = OwnPublication(receipt.0.committed_product_publication().clone());
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
    if let Some(denial) = continuation_denial {
        // The effect already exists. Keep its receipt, delivery and prepaid
        // Ready backing in the ordinary retry/cleanup owner even on late stop.
        return Err(CompletionRefusal::unpublished(denial, checkpoint));
    }
    if let Err((denial, checkpoint)) = selected_execution_finish.publish(checkpoint) {
        return Err(CompletionRefusal::unpublished(denial, checkpoint));
    }
    Ok(CheckpointProgress::Published(publication))
}
