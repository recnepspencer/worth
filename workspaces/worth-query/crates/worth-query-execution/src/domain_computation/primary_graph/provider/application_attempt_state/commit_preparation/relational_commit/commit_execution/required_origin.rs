//! Bind complete installed-family publication at the actual precommit image.
use super::*;
mod authored_retirement;
use crate::domain_computation::primary_graph::PreparedPrerequisiteClaims;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::mvcc::RequiredOutputPublication;
use worth_relational::facade::mvcc::RequiredOutputPublicationMeaning;
use worth_relational::facade::mvcc::ValidatedRelationalProposal;

pub(super) fn bind(
    proposal: ValidatedRelationalProposal,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before: &worth_relational::facade::snapshots::SnapshotHandle,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    required: Option<&PreparedPrerequisiteClaims>,
    owned_history: bool,
) -> Result<
    ValidatedRelationalProposal,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    if proposal.has_coupled_authored_publication() {
        return Ok(proposal);
    }
    let Some(required) = required else {
        return authored_retirement::bind(proposal, execution, before, provider, attempt);
    };
    let Some(publisher) = &provider.graph.application_publication else {
        return Ok(proposal);
    };
    let Some(dependency) = attempt.idempotency().producer_dependency_identity() else {
        return authored_retirement::bind(proposal, execution, before, provider, attempt);
    };
    let mismatch = || native_output_witness_stop(CompanionPreflightStop::SelectedSourceMismatch);
    let Some(partition) = attempt.idempotency().source_partition_identity() else {
        return Err(mismatch());
    };
    let targets = attempt.native_publication_roles();
    if targets.len() == 0 {
        return Ok(proposal);
    }
    let overflow =
        || native_output_witness_stop(CompanionPreflightStop::PreparationMemoryCounterOverflow);
    let product = attempt.affinity().product_publication().observation();
    let native_basis = product.basis().relational_basis();
    provider.graph.with_runtime(execution, |runtime| {
        let positioned = runtime
            .read_truth()
            .positioned_snapshot(before)
            .map_err(|_| mismatch())?;
        if positioned.root_id() != native_basis.descriptor().root_identity()
            || positioned.runtime_instance_id() != native_basis.descriptor().runtime_instance_id()
            || positioned.branch_id() != native_basis.descriptor().branch_id()
        {
            return Err(mismatch());
        }
        Ok(())
    })?;
    let scope = attempt.affinity().operation_scope();
    let (family, head) = provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .required_publication_head(
            required.native_publication_family(),
            attempt.output_binding_type().ok_or_else(overflow)?,
            scope,
            product.lifecycle_incarnation(),
            product.reference_generation().get(),
            partition,
        )
        .map_err(native_output_witness_stop)?;
    let (producer, source_epoch) = required.native_prior_checkpoint_input();
    let (roles, all_created) =
        crate::domain_computation::primary_graph::provider::native_publication_roles::collect(
            &provider.graph.layout,
            targets,
        )
        .ok_or_else(overflow)?;
    let source_scope = scope.scope();
    let meaning = RequiredOutputPublicationMeaning {
        implementation: attempt
            .idempotency()
            .producing_implementation()
            .ok_or_else(mismatch)?,
        family,
        source_scope: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::new(source_scope.partition_id()),
            source_scope.local_slot(),
            source_scope.generation(),
        ),
        source_partition: partition,
        source_epoch,
    };
    let retired = provider
        .graph
        .with_runtime(execution, |runtime| {
            runtime.readmit_native_output_transition_successor(
                execution,
                &publisher.native,
                native_basis,
                &meaning,
            )
        })
        .map_err(transaction_commit_stop)?;
    let warm = required
        .selected_read_predecessor()
        .or(retired)
        .or_else(|| head.as_ref().and_then(|head| head.warm_publication()));
    // A cold Query memo may predate a later Native-owned publication. Native's
    // current roster owns succession; old checkpoint correspondence does not.
    let warm = match warm {
        Some(publication) => Some(
            provider
                .graph
                .with_runtime(execution, |runtime| {
                    runtime.readmit_carried_required_output_publication(
                        execution,
                        &publisher.native,
                        native_basis,
                        &publication,
                    )
                })
                .map_err(transaction_commit_stop)?,
        ),
        None => provider
            .graph
            .with_runtime(execution, |runtime| {
                runtime.readmit_current_owned_output_successor(
                    execution,
                    &publisher.native,
                    native_basis,
                    producer,
                    &meaning,
                )
            })
            .map_err(transaction_commit_stop)?,
    };
    // An owned selection can explicitly remove this route while Query still
    // retains the future's warm lineage. Only Native's exact absence custody
    // can authorize Birth at this image; that warm row cannot select succession.
    let absence = if warm.is_some() && !owned_history {
        None
    } else {
        provider
            .graph
            .with_runtime(execution, |runtime| {
                runtime.readmit_required_output_route_absence(
                    execution,
                    &publisher.native,
                    native_basis,
                    &meaning,
                )
            })
            .map_err(transaction_commit_stop)?
    };
    let warm = if absence.is_some() { None } else { warm };
    use crate::domain_computation::primary_graph::provider::publication_posture::{
        self, ProducerPublicationPosture as Posture,
    };
    let posture = publication_posture::select(absence, all_created);
    if matches!(posture, Posture::Ordinary(_)) {
        // A producer can preserve an authored carrier without owning its birth.
        // No existing role or created companion acquires lifetime ownership.
        return Ok(proposal);
    }
    let running = publisher.owner.admit_required_output(producer, dependency);
    let publication = if let Posture::Birth(absence) = posture {
        RequiredOutputPublication::Birth {
            admission: running,
            absence: Some(absence),
            meaning,
            roles,
        }
    } else {
        let predecessor = if let Some(native) = warm {
            native
        } else {
            let head = head.ok_or_else(mismatch)?;
            provider.graph.with_runtime(execution, |runtime| {
                head.readmit(runtime, execution, &publisher.native, native_basis,
                    &meaning.family, scope.scope(), partition, &provider.graph.layout)
            }).map_err(|stop| match stop {
                crate::domain_computation::primary_graph::output_lineage::required_publication::head::RequiredPublicationReadmissionStop::Selection(stop) => native_output_witness_stop(stop),
                crate::domain_computation::primary_graph::output_lineage::required_publication::head::RequiredPublicationReadmissionStop::Native(stop) => transaction_commit_stop(stop),
                crate::domain_computation::primary_graph::output_lineage::required_publication::head::RequiredPublicationReadmissionStop::Allocation(stop) => readmission_allocation(stop),
            })?.publication().clone()
        };
        let succession = publisher
            .owner
            .admit_required_output_succession(running, predecessor.authority())
            .ok_or_else(mismatch)?;
        RequiredOutputPublication::Preserve {
            admission: succession,
            predecessor,
            meaning,
            roles,
        }
    };
    proposal
        .bind_required_output_publication(&publisher.native, publication)
        .map_err(transaction_commit_stop)
}
