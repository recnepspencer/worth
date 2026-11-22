//! Full native validation owns its end-of-validation window before returning.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::ObservedSourcePublication;
use std::sync::Arc;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
pub(super) fn validate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    before: &worth_relational::facade::snapshots::SnapshotHandle,
    observations: &[Arc<ObservedSourcePublication>],
) -> Result<(), crate::domain_computation::WorthQueryProviderSessionCommitStop> {
    let stop = consumed_output::verification_stop;
    let map_admission=crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop;
    provider.graph.with_runtime(execution, |runtime| {
        for observed in observations {
            let fact = Fact::SourcePublication {
                observation: std::sync::Arc::clone(observed),
            };
            let current = fact
                .source_currentness_in(runtime, execution, before)
                .map_err(|cause| stop(map_admission(cause)))?
                .map_err(|cause| {
                    stop(match cause {
                        WorthQuerySourceCurrentnessFailure::ManagedIndex(cause) => {
                            VerificationStop::ManagedIndex(cause)
                        }
                        WorthQuerySourceCurrentnessFailure::OutputObservation(cause) => cause,
                        WorthQuerySourceCurrentnessFailure::Execution(cause) => {
                            VerificationStop::Execution(cause)
                        }
                        WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => {
                            VerificationStop::WorkExhausted
                        }
                        WorthQuerySourceCurrentnessFailure::Unavailable => {
                            VerificationStop::Unavailable
                        }
                    })
                })?;
            if current.movement() != Movement::Unmoved {
                return Err(stop(VerificationStop::Unavailable));
            }
        }
        Ok(())
    })?;
    // The owning validator does not return until this exact window has passed.
    // A late-capture implementation in prepare necessarily captures AFTER it.
    // The graph loan is released so the actual other producer may complete.
    let control = &provider.application_attempt_operation_control;
    let observed = control.source_validation_observed();
    if observed {
        control.record_source_validation_image(image(execution, provider, before, observations)?);
    }
    control.after_source_publication_validation();
    if observed {
        control.record_source_validation_image(image(execution, provider, before, observations)?);
    }
    Ok(())
}
fn image(execution:&worth_execution::ExecutionRequest<'_, '_>,provider:&WorthQueryPrimaryGraphProvider,
 before:&worth_relational::facade::snapshots::SnapshotHandle,observations:&[Arc<ObservedSourcePublication>],
)->Result<crate::domain_computation::primary_graph::application_runtime::WorthQuerySourcePublicationImageForTest,crate::domain_computation::WorthQueryProviderSessionCommitStop>{
    let stop = consumed_output::verification_stop;
    provider.graph.with_runtime(execution,|runtime|{
 let selected=runtime.read_truth().positioned_snapshot(before).map_err(|_|stop(VerificationStop::Unavailable))?;
 source_publication_validation_image_for_test(&provider.graph.source_owner_ref().invalidation_owner, observations, &selected)
 .map_err(|cause|stop(crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop(cause)))
 })
}

fn source_publication_validation_image_for_test(
    owner: &crate::domain_computation::primary_graph::SourceInvalidationOwner,
    observations: &[Arc<ObservedSourcePublication>],
    selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
) -> Result<
    crate::domain_computation::primary_graph::application_runtime::WorthQuerySourcePublicationImageForTest,
    worth_relational::facade::mvcc::CompanionPreflightStop,
>{
    use crate::domain_computation::primary_graph::application_runtime::{
        WorthQuerySourcePublicationHeadForTest as Head,
        WorthQuerySourcePublicationImageForTest as View,
    };
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    let cell = owner
        .cell_for_read(selected)?
        .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
    let image = cell.read_image();
    let mut outer_publications = Vec::with_capacity(observations.len());
    for observed in observations {
        let state = image
            .payload()
            .current
            .source_publications
            .get(&observed.address)
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        let mut heads = Vec::with_capacity(state.heads.len());
        for head in state.heads.iter() {
            let (occurrence, generation, slot) = head.identity.address();
            heads.push(Head {
                binding: head.key.source.output_binding.as_str().to_owned(),
                partition: head.key.partition,
                occurrence,
                generation,
                slot,
            });
        }
        outer_publications.push(heads);
    }
    Ok(View {
        root_id: image.root_id(),
        native_commit: image.commit_id().map(|id| id.0),
        native_position: image.position().map(|position| position.0),
        topology_generation: image.topology_generation(),
        outer_publications,
    })
}
