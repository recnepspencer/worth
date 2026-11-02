//! Catalog-bounded closure from installed source ownership, never submitted reachability.
mod roster;
use super::{charge, require_linear_history, CodecRetirementClosureStop as Stop};
use crate::storage::overlay::PartitionAccess;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::{CommitId, SelectedOutputPublication},
    identity::data::EntityId,
    mvcc::{
        RelationalInstalledApplicationPublicationAuthority, RequiredOutputRolePosture as Posture,
        RequiredOutputRoleTarget,
    },
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    runtime::RelationalRuntime,
    transactions::data::{EntityReference, RecordRef, TransactionCommitError},
};
use roster::require_roster;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, MapKernelContext};

pub(super) fn reject(cause: Cause, id: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        cause,
        RecordRef::Entity(id),
    ))
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

/// O(H² + P²M + R² + PMR), bounded by the actual catalog H, latest publications P,
/// their members M and submitted roles R. Every traversal/comparison is charged.
pub(crate) fn select_authored_source_retirement(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    anchor: EntityId,
    roles: &[RequiredOutputRoleTarget],
) -> Result<(CommitId, Vec<SelectedOutputPublication>), TransactionCommitError> {
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                if installed.runtime_instance_id != runtime.runtime_instance_id()
                    || basis.descriptor().runtime_instance_id() != runtime.runtime_instance_id()
                    || !basis.is_current()
                {
                    return Err(reject(Cause::InvalidAuthoredAnchor, anchor));
                }
                let birth = authored_birth(execution, runtime, basis, anchor, context)?;
                let bound = runtime.history.catalog_len();
                let mut latest: Vec<SelectedOutputPublication> = Vec::with_capacity(bound);
                let mut next = basis.inner.root.commit_id();
                for _ in 0..bound {
                    let Some(commit) = next else { break };
                    charge(context, bound.saturating_add(1)).map_err(stopped)?;
                    let envelope = runtime
                        .history
                        .canonical_envelope(commit)
                        .ok_or_else(|| reject(Cause::CorruptPublication, anchor))?;
                    require_linear_history(&envelope)
                        .map_err(|_| reject(Cause::CorruptPublication, anchor))?;
                    for (ordinal, origin) in envelope.output_origins().enumerate() {
                        charge(context, latest.len() + 1).map_err(stopped)?;
                        let work = latest.iter().fold(origin.members.len(), |sum, prior| {
                            sum.saturating_add(origin.meaning.family.len())
                                .saturating_add(prior.origin.meaning.family.len() + 64)
                        });
                        charge(context, work).map_err(stopped)?;
                        envelope
                            .output_origin_for_route(&origin.meaning, context)
                            .map_err(|stop| match stop {
                                Stop::Request(_) => stopped(stop),
                                Stop::Payload(_) => {
                                    reject(Cause::CorruptPublication, origin.meaning.source_scope)
                                }
                            })?;
                        if !latest
                            .iter()
                            .any(|prior| prior.origin.meaning.same_route(&origin.meaning))
                        {
                            if latest.len() >= bound {
                                return Err(reject(Cause::CorruptPublication, anchor));
                            }
                            charge(
                                context,
                                installed
                                    .source_scope_ownership
                                    .lookup_work(&origin.producer),
                            )
                            .map_err(stopped)?;
                            if installed
                                .source_scope_ownership
                                .scope_kind(&origin.producer)
                                .is_some()
                                != origin.source_scope_owned
                            {
                                return Err(reject(
                                    Cause::CorruptPublication,
                                    origin.meaning.source_scope,
                                ));
                            }
                            latest.push(SelectedOutputPublication {
                                envelope: Arc::clone(&envelope),
                                origin: Arc::clone(origin),
                                ordinal,
                            });
                        }
                    }
                    next = if let Some(event) = &envelope.required_output_materialization {
                        super::materialization::validate_semantics(
                            runtime, &envelope, context, lease,
                        )
                        .map_err(stopped)?;
                        Some(event.source_commit)
                    } else {
                        envelope.commit.parents.first().copied()
                    };
                }
                if next.is_some() {
                    return Err(reject(Cause::CorruptPublication, anchor));
                }
                let mut selected: Vec<SelectedOutputPublication> = Vec::with_capacity(latest.len());
                for _ in 0..=latest.len() {
                    let mut advanced = false;
                    for envelope in &latest {
                        let origin = &envelope.origin;
                        charge(
                            context,
                            selected.len() + origin.members.len() + origin.producer.len() + 1,
                        )
                        .map_err(stopped)?;
                        if selected
                            .iter()
                            .any(|prior| Arc::ptr_eq(&prior.origin, &envelope.origin))
                            || !origin.source_scope_owned
                            || origin.members.iter().all(|m| m.posture == Posture::Retire)
                        {
                            continue;
                        }
                        let source = origin.meaning.source_scope;
                        let mut in_closure = source == anchor;
                        for prior in &selected {
                            let members = &prior.origin.members;
                            charge(context, members.len()).map_err(stopped)?;
                            in_closure |= members
                                .iter()
                                .any(|m| m.id == source && m.posture != Posture::Retire);
                        }
                        if !in_closure {
                            continue;
                        }
                        charge(
                            context,
                            installed
                                .source_scope_ownership
                                .lookup_work(&origin.producer),
                        )
                        .map_err(stopped)?;
                        let kind = installed
                            .source_scope_ownership
                            .scope_kind(&origin.producer)
                            .ok_or_else(|| reject(Cause::CorruptPublication, source))?;
                        basis
                            .inner
                            .root
                            .demand_full_partition(execution, source.partition_id)
                            .map_err(|cause| stopped(super::request(cause)))?;
                        let partition = basis
                            .inner
                            .root
                            .partition_state(source.partition_id)
                            .map_err(|cause| stopped(cause.into()))?
                            .ok_or_else(|| reject(Cause::CorruptPublication, source))?;
                        if !partition
                            .entity_arena
                            .get(&source)
                            .is_some_and(|row| row.is_live() && row.kind_id() == Some(kind))
                        {
                            return Err(reject(Cause::CorruptPublication, source));
                        }
                        selected.push(envelope.clone());
                        advanced = true;
                    }
                    if !advanced {
                        break;
                    }
                }
                if selected.is_empty() {
                    return Ok((birth, selected));
                }
                require_roster(anchor, roles, &selected, &latest, context)?;
                // Cover the exact fixed-width legality-seal comparisons performed by proof admission.
                charge(context, selected.len().saturating_mul(selected.len() + 1))
                    .map_err(stopped)?;
                charge(context, roles.len()).map_err(stopped)?;
                for role in roles
                    .iter()
                    .filter(|role| role.target == EntityReference::Existing(anchor))
                {
                    let partition = basis
                        .inner
                        .root
                        .partition_state(anchor.partition_id)
                        .map_err(|cause| stopped(cause.into()))?
                        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
                    if !partition
                        .entity_arena
                        .get(&anchor)
                        .is_some_and(|row| row.kind_id() == Some(role.kind))
                    {
                        return Err(reject(Cause::InvalidAuthoredAnchor, anchor));
                    }
                }
                Ok((birth, selected))
            },
        )
    });
    execution
        .consult()
        .map_err(|cause| stopped(super::request(cause)))?;
    result
        .map_err(|cause| stopped(super::request(cause)))?
        .map_err(|cause| stopped(super::request(cause)))?
        .0
}

fn authored_birth(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    anchor: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CommitId, TransactionCommitError> {
    use crate::publication::patch::data::RecordStructuralChange as Change;
    if anchor.generation_value() == 0 {
        return Err(reject(Cause::InvalidAuthoredAnchor, anchor));
    }

    basis
        .inner
        .root
        .demand_full_partition(execution, anchor.partition_id)
        .map_err(|cause| stopped(super::request(cause)))?;
    let partition = basis
        .inner
        .root
        .partition_state(anchor.partition_id)
        .map_err(|cause| stopped(Stop::from(cause)))?
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    partition
        .entity_arena
        .get(&anchor)
        .filter(|row| row.is_live())
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    let created = partition
        .entity_arena
        .created_at_for_slot(anchor.slot_index())
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    let envelope = runtime
        .history
        .canonical_envelope_for_version(created)
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    charge(
        context,
        envelope.record_allocations.len() + envelope.patch.authoritative_record_patches.len() + 1,
    )
    .map_err(stopped)?;
    let mut owned = false;
    for origin in envelope.output_origins() {
        charge(context, origin.outputs.len()).map_err(stopped)?;
        owned |= origin.outputs.contains(&anchor);
    }
    if owned
        || envelope
            .record_allocations
            .iter()
            .filter(|allocation| allocation.record() == &RecordRef::Entity(anchor))
            .count()
            != 1
        || !envelope
            .patch
            .authoritative_record_patches
            .iter()
            .any(|patch| {
                patch.target == RecordRef::Entity(anchor)
                    && patch.structural_change == Change::Created
            })
    {
        return Err(reject(Cause::InvalidAuthoredAnchor, anchor));
    }
    Ok(envelope.commit.commit_id)
}
