//! The materialization owner captures complete native facts before payload disappears.
use super::*;
use crate::branch::{
    AdmittedRelationalBranchBasis, RelationalMaterializationCustody,
    RelationalMaterializationRecord,
};
use crate::history::data::{
    CanonicalRequiredOutputMaterialization as Event,
    CanonicalRequiredOutputMaterializationRecord as Member,
};
use crate::runtime::RelationalRuntime;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::TransactionCommitError;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, ExecutionWorkCeiling};

mod selection;

pub(crate) fn capture_materialization_producers(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    manifest: &[RelationalMaterializationRecord],
) -> Result<(Vec<CommitId>, Vec<Member>, ExecutionMemoryReservation), TransactionCommitError> {
    if basis.descriptor().runtime_instance_id() != runtime.runtime_instance_id()
        || !basis.is_current()
    {
        return Err(crate::mvcc::origin_denial(
            "materialization capture requires exact current native source basis",
        ));
    }
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
                    let mut publications = Vec::with_capacity(manifest.len());
                    let mut records = Vec::with_capacity(manifest.len());
                    for record in manifest {
                        let selected =
                            match selection::at_source(runtime, basis, record, context, lease) {
                                Ok(Some(selected)) => selected,
                                Ok(None) => continue,
                                Err(cause) => return Err(cause),
                            };
                        let origin = selected
                            .validate_output_membership_header()
                            .map_err(|cause| stop(cause.into()))?;
                        charge(context, publications.len() + 1).map_err(stop)?;
                        if publications.contains(&origin.commit.commit_id) {
                            continue;
                        }
                        let complete = (|| {
                            for member in &origin.members {
                                let partition = basis
                                    .inner
                                    .root
                                    .partition_state(member.id.partition_id)
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
                                    row.extra().authoritative_aspect_state.as_ref(),
                                    context,
                                    lease,
                                )?;
                            }
                            let origin = Arc::clone(
                                selected
                                    .required_output_origin
                                    .as_ref()
                                    .ok_or("producer association absent")?,
                            );
                            super::super::relations::validate_complete_publication(
                                runtime,
                                &basis.inner.root,
                                &origin,
                                context,
                                lease,
                            )
                        })();
                        match complete {
                            Ok(()) => publications.push(origin.commit.commit_id),
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
                        if !publications.iter().any(|publication| {
                            runtime
                                .history
                                .canonical_envelope(*publication)
                                .and_then(|envelope| {
                                    envelope
                                        .required_output_origin
                                        .as_ref()
                                        .map(|origin| header::owns(origin, &record))
                                })
                                .unwrap_or(false)
                        }) {
                            continue;
                        }
                        let before_write = match record {
                            RecordRef::Entity(id) => basis
                                .inner
                                .root
                                .partition_state(id.partition_id)
                                .and_then(|part| {
                                    part.entity_arena.metadata_history_at(id.slot_index())
                                })
                                .and_then(|history| history.get(history.len().checked_sub(1)?))
                                .map(|entry| entry.effective_at),
                            RecordRef::Relation(id) => basis
                                .inner
                                .root
                                .partition_state(id.partition_id)
                                .and_then(|part| {
                                    part.relation_arena.metadata_history_at(id.slot_index())
                                })
                                .and_then(|history| history.get(history.len().checked_sub(1)?))
                                .map(|entry| entry.effective_at),
                        }
                        .ok_or_else(|| {
                            crate::mvcc::origin_denial(
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

pub(crate) fn prepare_materialization_continuity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    custody: &RelationalMaterializationCustody,
    candidate: &WorkingState,
) -> Result<Option<Arc<Event>>, TransactionCommitError> {
    let captured = custody.data();
    if captured.producer_publications.is_empty() {
        return Ok(None);
    }
    let result = execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    let mut publications = Vec::with_capacity(captured.producer_publications.len());
                    for publication in &captured.producer_publications {
                        charge(context, 1)?;
                        let original = runtime
                            .history
                            .canonical_envelope(*publication)
                            .ok_or("materialization original producer history absent")?;
                        if payload::candidate_matches(
                            runtime,
                            basis,
                            candidate,
                            &original,
                            &captured.producer_records,
                            context,
                            lease,
                        )? {
                            publications.push(*publication);
                        }
                    }
                    if publications.is_empty() {
                        return Ok(None);
                    }
                    let mut records = Vec::with_capacity(captured.producer_records.len());
                    for record in &captured.producer_records {
                        charge(context, publications.len() + 1)?;
                        if publications.iter().any(|publication| {
                            runtime
                                .history
                                .canonical_envelope(*publication)
                                .and_then(|envelope| {
                                    envelope
                                        .required_output_origin
                                        .as_ref()
                                        .map(|origin| header::owns(origin, &record.record))
                                })
                                .unwrap_or(false)
                        }) {
                            records.push(record.clone());
                        }
                    }
                    Ok::<_, Stop>(Some(Arc::new(Event {
                        source_commit: captured.source_commit,
                        suspension_commit: captured.suspension_commit,
                        producer_publications: publications,
                        records,
                    })))
                },
            )
        })
        .map_err(|cause| stop(Stop::Request(cause.into())))?
        .map_err(|cause| stop(Stop::Request(cause.into())))?
        .0
        .map_err(stop)?;
    execution
        .consult()
        .map_err(|cause| stop(Stop::Request(cause.into())))?;
    Ok(result)
}

fn stop(cause: Stop) -> TransactionCommitError {
    match cause {
        Stop::Payload(detail) => crate::mvcc::origin_denial(detail),
        Stop::Request(cause) => {
            TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            })
        }
    }
}
