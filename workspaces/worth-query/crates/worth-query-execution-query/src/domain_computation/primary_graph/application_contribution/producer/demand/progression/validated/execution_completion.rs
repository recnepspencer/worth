//! Finish the prepaid row publication on the canonical application owner.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::execution;
use crate::domain_computation::primary_graph::application_output_demand::RunningSelectedExecution;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint;

#[allow(clippy::too_many_arguments)]
pub struct CompletionRefusal {
    pub denial: WorthQueryOutputDemandDenial,
    pub checkpoint: Option<WorthQueryOutputCheckpoint>,
}
impl CompletionRefusal {
    pub fn denied(denial: WorthQueryOutputDemandDenial) -> Self {
        Self {
            denial,
            checkpoint: None,
        }
    }
    pub fn unpublished(
        denial: WorthQueryOutputDemandDenial,
        checkpoint: WorthQueryOutputCheckpoint,
    ) -> Self {
        Self {
            denial,
            checkpoint: Some(checkpoint),
        }
    }
}

pub fn finish(
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
                owner: Box::new(owner),
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
        Ok((execution::ProducerExecutionOutcome::LiveCompletion(receipt), ready_backing)) => {
            // The original coupled publication already happened. Preserve its
            // existing Published delivery protocol without marking another
            // publication.
            performed.borrow_mut().mark_accepted(readiness);
            (receipt, ready_backing)
        }
        Ok((execution::ProducerExecutionOutcome::Stable(stable), ready_backing)) => {
            performed.borrow_mut().mark_accepted(readiness);
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
    publish_completed_receipt(
        receipt,
        ready_backing,
        selected_execution_finish,
        original_publication,
        execution,
    )
}

#[inline(never)]
fn publish_completed_receipt(
    mut receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ready_backing: crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
    selected_execution_finish: RunningSelectedExecution<'_>,
    original_publication: Option<std::sync::Arc<
        crate::domain_computation::primary_graph::application_output_demand::OriginalProducingPublication,
    >>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<CheckpointProgress, CompletionRefusal> {
    let retained = crate::domain_computation::primary_graph::application_output_demand::OriginalProducingPublication::for_successor(
        execution, original_publication, (&receipt).into(),
    );
    let (original_publication, continuation_denial) = match retained {
        Ok(original) => (original, None),
        Err(denial) => (None, Some(denial)),
    };
    let ready_backing = ready_backing.bind_original_producing_publication(original_publication);
    own_effect_observation::record(Stage::Published);
    let publication = OwnPublication(receipt.committed_product_publication().clone());
    let delivery = receipt
            .take_performed_relational_product_change()
            .map_or(
                crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery::NoChange,
                crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery::Change,
            );
    let checkpoint = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint::Published {
            receipt,
            delivery,
            ready_backing,
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
