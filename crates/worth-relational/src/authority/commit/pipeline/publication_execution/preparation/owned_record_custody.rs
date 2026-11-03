//! Completed native publications alone discharge custody for record destruction.
use crate::storage::overlay::{CurrentPartitionAccess, PartitionAccess};
use crate::{
    history::data::{CanonicalCommitEnvelope, CodecRetirementClosureStop as Stop},
    publication::{
        data::{AuthoredRetirementRefusal, PublicationError},
        patch::data::RecordStructuralChange,
    },
    runtime::RelationalPreparationRuntime,
    transactions::data::{RecordRef, TransactionCommitError},
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};

pub(super) fn require(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    candidate: &CanonicalCommitEnvelope,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
) -> Result<(), TransactionCommitError> {
    let checked = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                for patch in &candidate.patch.authoritative_record_patches {
                    context
                        .checkpoint(1)
                        .map_err(|cause| stopped(Stop::Request(cause.into())))?;
                    if !matches!(
                        patch.structural_change,
                        RecordStructuralChange::Deleted | RecordStructuralChange::RetainedForAudit
                    ) {
                        continue;
                    }
                    let origin = crate::history::data::select_owned_scope_dependent(
                        crate::history::data::OwnedScopeDependentSelection::LiveRecord(
                            &mut |origin, context| {
                                let mut covered = false;
                                for successor in candidate.output_origins() {
                                    context
                                        .checkpoint(
                                            (origin.meaning.family.len()
                                                + successor.meaning.family.len()
                                                + 65)
                                                as u64,
                                        )
                                        .map_err(|cause| Stop::Request(cause.into()))?;
                                    covered |= successor.meaning.same_route(&origin.meaning)
                                        && successor.publication.predecessor().is_some();
                                }
                                context
                                    .checkpoint(
                                        (origin.members.len() + origin.relations.len() + 1) as u64,
                                    )
                                    .map_err(|cause| Stop::Request(cause.into()))?;
                                if covered {
                                    return Ok(false);
                                }
                                let root = &basis.inner.root;
                                let live_entity =
                                    |id: crate::identity::data::EntityId| -> Result<bool, Stop> {
                                        root.demand_full_partition(execution, id.partition_id)
                                            .map_err(|cause| Stop::Request(cause.into()))?;
                                        Ok(root
                                            .get_current_partition(id.partition_id)
                                            .and_then(|partition| partition.entity_arena().get(&id))
                                            .is_some_and(|row| row.is_live()))
                                    };
                                let live_edge =
                                    |id: crate::identity::data::RelationId| -> Result<bool, Stop> {
                                        root.demand_full_partition(execution, id.partition_id)
                                            .map_err(|cause| Stop::Request(cause.into()))?;
                                        Ok(root
                                            .get_current_partition(id.partition_id)
                                            .and_then(|partition| {
                                                partition.relation_arena().get(&id)
                                            })
                                            .is_some_and(|row| row.is_live()))
                                    };
                                let mut matched = false;
                                match patch.target {
                                    RecordRef::Entity(id) => {
                                        if origin.meaning.source_scope == id {
                                            for member in &origin.members {
                                                if member.posture
                                                != crate::mvcc::RequiredOutputRolePosture::Retire
                                                && live_entity(member.id)?
                                            {
                                                matched = true;
                                                break;
                                            }
                                            }
                                        }
                                        matched |= origin.members.iter().any(|member| {
                                            member.id == id
                                            && member.posture
                                                != crate::mvcc::RequiredOutputRolePosture::Retire
                                        }) && live_entity(id)?;
                                        for edge in &origin.relations {
                                            if (edge.source == id || edge.target == id)
                                                && live_edge(edge.id)?
                                            {
                                                matched = true;
                                            }
                                        }
                                    }
                                    RecordRef::Relation(id) => {
                                        matched = origin.relations.iter().any(|edge| edge.id == id)
                                            && live_edge(id)?
                                    }
                                }
                                Ok(matched)
                            },
                        ),
                        candidate.commit.parents.first().copied(),
                        runtime.history.canonical_catalog_len(),
                        |commit| runtime.history.canonical_envelope(commit),
                        context,
                    )
                    .map_err(|failure| match failure.stop {
                        Stop::Request(_) => stopped(failure.stop),
                        Stop::Payload(_) => TransactionCommitError::publication(
                            PublicationError::authored_retirement(
                                AuthoredRetirementRefusal::CorruptPublication,
                                failure
                                    .source
                                    .map(RecordRef::Entity)
                                    .unwrap_or_else(|| patch.target.clone()),
                            ),
                        ),
                    })?;
                    if let Some(origin) = origin {
                        let owner = origin.meaning.source_scope;
                        let mut error = PublicationError::authored_retirement(
                            AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                                publication: origin.commit.commit_id,
                                owner,
                            },
                            patch.target.clone(),
                        );
                        if patch.target != RecordRef::Entity(owner) {
                            error
                                .context
                                .affected_records
                                .push(RecordRef::Entity(owner));
                        }
                        return Err(TransactionCommitError::publication(error));
                    }
                }
                Ok(())
            },
        )
    });
    execution
        .consult()
        .map_err(|cause| stopped(Stop::Request(cause.into())))?;
    checked
        .map_err(|cause| stopped(Stop::Request(cause.into())))?
        .map_err(|cause| stopped(Stop::Request(cause.into())))?
        .0
}

fn stopped(stop: Stop) -> TransactionCommitError {
    match stop {
        Stop::Payload(detail) => crate::mvcc::origin_denial(detail),
        Stop::Request(cause) => {
            TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            })
        }
    }
}
