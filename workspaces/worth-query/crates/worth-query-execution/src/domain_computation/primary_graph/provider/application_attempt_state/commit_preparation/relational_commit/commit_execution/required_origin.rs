//! Bind complete installed-family publication at the actual precommit image.
use super::*;
use crate::domain_computation::primary_graph::PreparedPrerequisiteClaims;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture as Posture;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::mvcc::RequiredOutputPublication;
use worth_relational::facade::mvcc::RequiredOutputPublicationMeaning;
use worth_relational::facade::mvcc::RequiredOutputRolePosture;
use worth_relational::facade::mvcc::RequiredOutputRoleTarget;
use worth_relational::facade::mvcc::ValidatedRelationalProposal;
use worth_relational::facade::transactions::EntityReference;

pub(super) fn bind(
    proposal: ValidatedRelationalProposal,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before: &worth_relational::facade::snapshots::SnapshotHandle,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    required: Option<&PreparedPrerequisiteClaims>,
) -> Result<
    ValidatedRelationalProposal,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let Some(required) = required else {
        return Ok(proposal);
    };
    let Some(publisher) = &provider.graph.application_publication else {
        return Ok(proposal);
    };
    let Some(dependency) = attempt.idempotency().producer_dependency_identity() else {
        return Ok(proposal);
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
    let mut roles = Vec::with_capacity(targets.len());
    let mut all_created = true;
    for (role, posture, name, target) in targets {
        let kind = provider
            .graph
            .layout
            .entity_kind(name)
            .ok_or_else(overflow)?;
        all_created &= posture == Posture::Create;
        roles.push(RequiredOutputRoleTarget {
            role: role.to_owned(),
            entity_name: name.to_owned(),
            kind,
            posture: match posture {
                Posture::Create => RequiredOutputRolePosture::Create,
                Posture::Preserve => RequiredOutputRolePosture::Preserve,
                Posture::Retire => RequiredOutputRolePosture::Retire,
            },
            target: target.clone(),
        });
    }
    let source_scope = scope.scope();
    let meaning = RequiredOutputPublicationMeaning {
        family,
        source_scope: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::new(source_scope.partition_id()),
            source_scope.local_slot(),
            source_scope.generation(),
        ),
        source_partition: partition,
        source_epoch,
    };
    let warm = head.as_ref().and_then(|head| head.warm_publication());
    let absence = if warm.is_some() {
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
    if absence.is_some() && !all_created {
        // An ordinary producer may preserve an authored carrier without ever
        // owning its birth. Native has established that this route has no
        // publication to succeed. Keep the whole operation ordinary: neither
        // existing roles nor any created companions acquire ownership here.
        return Ok(proposal);
    }
    let running = publisher.owner.admit_required_output(producer, dependency);
    let publication = if let Some(absence) = absence {
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
            let prior = head
                .ok_or_else(mismatch)?
                .cold_predecessor()
                .map_err(native_output_witness_stop)?;
            let views = prior.correspondence.native_witness_roles();

            let mut prior_roles = Vec::with_capacity(views.len());
            for (role, posture, name, id) in views {
                prior_roles.push(RequiredOutputRoleTarget {
                    role: role.to_owned(),
                    entity_name: name.to_owned(),
                    kind: provider
                        .graph
                        .layout
                        .entity_kind(name)
                        .ok_or_else(overflow)?,
                    posture: match posture {
                        Posture::Create => RequiredOutputRolePosture::Create,
                        Posture::Preserve => RequiredOutputRolePosture::Preserve,
                        Posture::Retire => RequiredOutputRolePosture::Retire,
                    },
                    target: EntityReference::Existing(id),
                });
            }

            // This is the performed epoch, not the new demand's source epoch.
            let prior_meaning = RequiredOutputPublicationMeaning {
                family: meaning.family.clone(),
                source_epoch: prior.source_epoch,
                source_scope: meaning.source_scope,
                source_partition: meaning.source_partition,
            };
            provider
                .graph
                .with_runtime(execution, |runtime| {
                    runtime.readmit_required_output_publication(
                        execution,
                        &publisher.native,
                        native_basis,
                        &prior.producer,
                        &prior.dependency,
                        &prior_meaning,
                        &prior_roles,
                    )
                })
                .map_err(transaction_commit_stop)?
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
