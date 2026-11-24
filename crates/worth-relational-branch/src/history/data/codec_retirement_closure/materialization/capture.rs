//! The materialization owner captures complete native facts before payload disappears.
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::materialization::RelationalMaterializationRecord;
use crate::history::data::canonical_commit_envelope::OutputOriginHistory;
use crate::history::data::CanonicalRequiredOutputMaterializationRecord as Member;
use crate::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, ExecutionWorkCeiling};

pub mod selection;

pub fn capture_materialization_producers<'r>(
    execution: &ExecutionRequest<'_, '_>,
    runtime: impl Into<
        crate::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs<
            'r,
        >,
    >,
    basis: &AdmittedRelationalBranchBasis,
    manifest: &[RelationalMaterializationRecord],
) -> Result<(Vec<CommitId>, Vec<Member>, ExecutionMemoryReservation), TransactionCommitError> {
    let runtime = &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest::new(runtime, execution);
    if basis.descriptor().runtime_instance_id() != runtime.input.projection.runtime_identity
        || !basis.is_current()
    {
        return Err(
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "materialization capture requires exact current native source basis",
            ),
        );
    }
    let regions = manifest
        .iter()
        .map(|record| match record {
            RelationalMaterializationRecord::Entity { entity_id, .. } => entity_id.partition_id,
            RelationalMaterializationRecord::Relation { relation_id, .. } => {
                relation_id.partition_id
            }
        })
        .collect::<std::collections::BTreeSet<_>>();
    basis
        .inner
        .root
        .demand_regions_full(execution, runtime.input.projection.symbols, regions)
        .map_err(|failure| stop(Stop::Request(failure.into())))?;
    execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    let bytes = manifest
                        .len()
                        .checked_mul(
                            std::mem::size_of::<CommitId>() + std::mem::size_of::<Member>(),
                        )
                        .ok_or_else(|| stop(super::super::memory_overflow()))?;
                    let memory = super::super::reserve(lease, bytes).map_err(stop)?;
                    let _groups_memory = super::super::reserve(
                        lease,
                        manifest
                            .len()
                            .checked_mul(std::mem::size_of::<
                                Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
                            >())
                            .ok_or_else(super::super::memory_overflow)
                            .map_err(stop)?,
                    )
                    .map_err(stop)?;
                    let mut complete_groups: Vec<
                        Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
                    > = Vec::with_capacity(manifest.len());
                    let mut publications = Vec::with_capacity(manifest.len());
                    let mut records = Vec::with_capacity(manifest.len());
                    let mut origin_frames = OutputOriginHistory::default();
                    let validated_history = &mut origin_frames;
                    for record in manifest {
                        let selected = match selection::at_source(
                            runtime,
                            basis,
                            record,
                            validated_history,
                            context,
                            lease,
                        ) {
                            Ok(Some(selected)) => selected,
                            Ok(None) => continue,
                            Err(cause) => return Err(cause),
                        };
                        let native_record = match record {
                            RelationalMaterializationRecord::Entity { entity_id, .. } => {
                                RecordRef::Entity(*entity_id)
                            }
                            RelationalMaterializationRecord::Relation { relation_id, .. } => {
                                RecordRef::Relation(*relation_id)
                            }
                        };
                        let origin = selected
                            .materialization_origin_for_record(&native_record, context)
                            .map_err(stop)?;
                        charge(context, complete_groups.len() + publications.len() + 1)
                            .map_err(stop)?;
                        if complete_groups
                            .iter()
                            .any(|prior| Arc::ptr_eq(prior, origin))
                        {
                            continue;
                        }
                        let complete = (|| {
                            for member in &origin.members {
                                let partition = basis
                                    .inner
                                    .root
                                    .partition_state(member.id.partition_id)?
                                    .ok_or("producer member partition absent")?;
                                let row = partition
                                    .entity_arena
                                    .get(&member.id)
                                    .ok_or("producer member generation absent")?;
                                super::super::membership::validate_member(
                                    runtime,
                                    basis,
                                    &selected,
                                    member.id,
                                    row.extra().authoritative_aspect_state(),
                                    validated_history,
                                    context,
                                    lease,
                                )?;
                            }
                            let origin = Arc::clone(origin);
                            super::super::relations::validate_complete_publication(
                                runtime,
                                &basis.inner.root,
                                &origin,
                                validated_history,
                                context,
                                lease,
                            )
                        })();
                        match complete {
                            Ok(()) => {
                                if !publications.contains(&origin.commit.commit_id) {
                                    publications.push(origin.commit.commit_id);
                                }
                                complete_groups.push(Arc::clone(origin));
                            }
                            Err(Stop::Payload(_)) => {}
                            Err(cause) => return Err(stop(cause)),
                        }
                    }
                    publications.sort_unstable();
                    for native in manifest {
                        let record = match native {
                            RelationalMaterializationRecord::Entity { entity_id, .. } => {
                                RecordRef::Entity(*entity_id)
                            }
                            RelationalMaterializationRecord::Relation { relation_id, .. } => {
                                RecordRef::Relation(*relation_id)
                            }
                        };
                        charge(context, publications.len() + 1).map_err(stop)?;
                        let mut owned = false;
                        for origin in &complete_groups {
                            charge(context, origin.members.len() + origin.relations.len() + 1)
                                .map_err(stop)?;
                            if header::owns(origin, &record) {
                                owned = true;
                                break;
                            }
                        }
                        if !owned {
                            continue;
                        }
                        let before_write = match record {
                            RecordRef::Entity(id) => basis
                                .inner
                                .root
                                .partition_state(id.partition_id)
                                .map_err(|failure| stop(Stop::from(failure)))?
                                .and_then(|part| {
                                    part.entity_arena.metadata_history_at(id.slot_index())
                                })
                                .and_then(|history| history.get(history.len().checked_sub(1)?))
                                .map(|entry| entry.effective_at),
                            RecordRef::Relation(id) => basis
                                .inner
                                .root
                                .partition_state(id.partition_id)
                                .map_err(|failure| stop(Stop::from(failure)))?
                                .and_then(|part| {
                                    part.relation_arena.metadata_history_at(id.slot_index())
                                })
                                .and_then(|history| history.get(history.len().checked_sub(1)?))
                                .map(|entry| entry.effective_at),
                        }
                        .ok_or_else(|| {
                            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                                "materialization producer native write absent",
                            )
                        })?;
                        records.push(Member {
                            record,
                            before_write,
                        });
                    }
                    records.sort_by(|left, right| left.record.cmp(&right.record));
                    Ok::<_, TransactionCommitError>((publications, records, memory))
                },
            )
        })
        .map_err(|cause| stop(Stop::Request(cause.into())))?
        .map_err(|cause| stop(Stop::Request(cause.into())))?
        .0
}

pub(super) fn stop(cause: Stop) -> TransactionCommitError {
    match cause {
        Stop::Payload(detail) => crate::transactions::data::outcomes::transaction_commit_error::origin_denial(detail),
        Stop::Request(cause) => {
            TransactionCommitError::execution(crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            })
        }
    }
}
