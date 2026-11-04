//! Complete membership selection is independent of each member's latest write.
use super::{
    charge, intent_writes_entity, require_linear_history, reserve,
    CodecRetirementClosureStop as Stop,
};
use crate::history::data::canonical_commit_envelope::{
    CanonicalOutputPublication, CanonicalRequiredOutputOrigin,
};
use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;
use crate::{
    branch::AdmittedRelationalBranchBasis, identity::data::EntityId, runtime::RelationalRuntime,
};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(super) fn latest(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    id: EntityId,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Arc<CanonicalCommitEnvelope>, Stop> {
    let mut next = basis.inner.root.commit_id();
    let bound = runtime.history.catalog_len();
    for _ in 0..bound {
        charge(context, bound.saturating_add(1))?;
        let commit = next.ok_or("output correspondence has no complete native publication")?;
        let envelope = runtime
            .history
            .canonical_envelope(commit)
            .ok_or("output correspondence lost native history")?;
        require_linear_history(&envelope)?;
        for origin in envelope.output_origins() {
            charge(context, origin.outputs.len())?;
            if origin.outputs.binary_search(&id).is_ok() {
                envelope.output_origin_for_entity(id, context)?;
                return Ok(Arc::clone(&envelope));
            }
        }
        if writes(&envelope, id, context)? {
            if let Some(event) = &envelope.required_output_materialization {
                super::materialization::validate_semantics(
                    runtime,
                    &envelope,
                    context,
                    lease,
                    basis.inner.root.schema_authority(),
                )?;
                for publication in &event.producer_publications {
                    let original = runtime
                        .history
                        .canonical_envelope(*publication)
                        .ok_or("maintenance producer history absent")?;
                    let origin = original.validate_output_membership_header()?;
                    if origin.outputs.binary_search(&id).is_ok() {
                        return Ok(original);
                    }
                }
            }
            return Err("present output has no installed producer publication".into());
        }
        next = envelope.commit.parents.first().copied();
    }
    Err("output correspondence ancestry is incomplete or cyclic".into())
}

pub(super) fn validate_member(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    selected: &Arc<CanonicalCommitEnvelope>,
    id: EntityId,
    payload: Option<&AuthoritativeRecordAspectState>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Arc<CanonicalRequiredOutputOrigin>, Stop> {
    super::restoration_payload::validate_groups(
        runtime,
        selected,
        context,
        basis.inner.root.schema_authority(),
    )?;
    let origin = selected.output_origin_for_entity(id, context)?;
    let member = origin
        .members
        .iter()
        .find(|member| member.id == id)
        .ok_or("publication omits exact member")?;
    let partition = basis
        .inner
        .root
        .partition_state(id.partition_id)?
        .ok_or("output partition absent")?;
    let row = partition
        .entity_arena
        .get(&id)
        .ok_or("output generation absent or retired")?;
    let write = if member.posture == crate::mvcc::RequiredOutputRolePosture::Retire {
        row.retired_at()
            .ok_or("output lacks its actual retirement position")?
    } else {
        partition
            .entity_arena
            .metadata_history_at(id.slot_index())
            .and_then(|rows| rows.get(rows.len().checked_sub(1)?))
            .ok_or("output native write position absent")?
            .effective_at
    };
    if row.is_live() != (member.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
        || row.kind_id() != Some(member.kind)
        || !super::materialization::write_matches(
            &RecordRef::Entity(id),
            selected.commit.commit_id,
            member.after_write,
            write,
            &mut |commit| runtime.history.canonical_envelope(commit),
            &mut |version| runtime.history.canonical_envelope_for_version(version),
            runtime.history.catalog_len(),
            context,
        )?
    {
        return Err("output differs from its complete publication native write position".into());
    }
    let schema = basis.inner.root.schema_authority();
    charge(
        context,
        (usize::BITS - schema.entity_aspect_plan_count().leading_zeros()) as usize + 1,
    )?;
    let plan = schema
        .entity_aspect_plan(member.kind)
        .ok_or("output physical contract plan absent")?;
    let reconstructed = reconstruct(
        runtime,
        selected,
        id,
        plan,
        context,
        lease,
        basis.inner.root.schema_authority(),
    )?;
    charge(
        context,
        payload.map_or(0, |payload| payload.owned_allocation_capacity_bytes()),
    )?;
    if member.posture != crate::mvcc::RequiredOutputRolePosture::Retire
        && reconstructed
            .as_ref()
            .and_then(|state| state.state.as_ref())
            != payload
    {
        return Err("present payload differs from native producer patch succession".into());
    }
    Ok(Arc::clone(selected.output_origin_for_entity(id, context)?))
}

pub(super) fn reconstruct(
    runtime: &RelationalRuntime,
    selected: &Arc<CanonicalCommitEnvelope>,
    id: EntityId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<Option<super::payload_reconstruction::ReconstructedPayload>, Stop> {
    reconstruct_chain(runtime, selected, id, plan, context, lease, true, schema)
}

/// Canonical producer patches supply expected bytes; their maintenance gaps
/// are validated by the enclosing semantic pass rather than recursively replayed.
pub(super) fn reconstruct_canonical_payload(
    runtime: &RelationalRuntime,
    selected: &Arc<CanonicalCommitEnvelope>,
    id: EntityId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<Option<super::payload_reconstruction::ReconstructedPayload>, Stop> {
    reconstruct_chain(runtime, selected, id, plan, context, lease, false, schema)
}

#[allow(clippy::too_many_arguments)]
fn reconstruct_chain(
    runtime: &RelationalRuntime,
    selected: &Arc<CanonicalCommitEnvelope>,
    id: EntityId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    validate_gap_payload: bool,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<Option<super::payload_reconstruction::ReconstructedPayload>, Stop> {
    super::restoration_payload::validate_groups(runtime, selected, context, schema)?;
    let origin = selected.output_origin_for_entity(id, context)?;
    let _member = origin
        .members
        .iter()
        .find(|member| member.id == id)
        .ok_or("publication omits exact member")?;
    let bound = runtime.history.catalog_len();
    let _chain_memory = reserve(
        lease,
        bound
            .checked_mul(std::mem::size_of::<Arc<CanonicalCommitEnvelope>>())
            .ok_or_else(super::memory_overflow)?,
    )?;
    let mut chain = Vec::with_capacity(bound);
    let mut current = Arc::clone(selected);
    loop {
        if chain.len() >= bound {
            return Err("publication succession is cyclic or incomplete".into());
        }
        charge(context, bound)?;
        let current_origin = current.output_origin_for_route(&origin.meaning, context)?;
        chain.push(Arc::clone(&current));
        match current_origin.publication {
            CanonicalOutputPublication::Birth
            | CanonicalOutputPublication::BirthAfterCodecRetirement { .. } => {
                if current_origin.birth != current.commit.commit_id
                    || current_origin.birth_producer != current_origin.producer
                    || current_origin.birth_dependency != current_origin.dependency_identity
                {
                    return Err("publication birth identity differs".into());
                }
                super::codec_disposition::validate_birth(
                    runtime, &current, context, lease, schema,
                )?;
                break;
            }
            CanonicalOutputPublication::AuthoredRestoration {
                retirement,
                predecessor,
            } => {
                super::restoration_payload::validate_groups(runtime, &current, context, schema)?;
                // A later Preserve can introduce a distinct member after this
                // restoration base. Its actual Create is verified during replay;
                // only the base's members have an original restored payload.
                if current_origin
                    .members
                    .binary_search_by_key(&id, |member| member.id)
                    .is_ok()
                {
                    super::restoration_payload::validate_member(
                        runtime,
                        &current,
                        id,
                        retirement,
                        predecessor,
                        plan,
                        context,
                        lease,
                        schema,
                    )?;
                }
                break;
            }
            CanonicalOutputPublication::Preserve { predecessor }
            | CanonicalOutputPublication::AuthoredRetirement { predecessor }
            | CanonicalOutputPublication::ProducerRetirement { predecessor } => {
                let previous = runtime
                    .history
                    .canonical_envelope(predecessor)
                    .ok_or("publication predecessor history missing")?;
                let prior = previous.output_origin_for_route(&current_origin.meaning, context)?;
                super::publication_succession::validate_succession(
                    runtime, &current, prior, context, schema,
                )?;
                super::publication_succession::validate_gap(
                    runtime,
                    &current,
                    predecessor,
                    current_origin,
                    context,
                    lease,
                    validate_gap_payload,
                    schema,
                )?;
                current = previous;
            }
        }
    }
    let mut reconstructed: Option<super::payload_reconstruction::ReconstructedPayload> = None;
    for envelope in chain.iter().rev() {
        let membership = envelope.output_origin_for_route(&origin.meaning, context)?;
        let member = membership.members.iter().find(|member| member.id == id);
        let Some(member) = member else { continue };
        let patch = envelope
            .patch
            .authoritative_record_patches
            .iter()
            .find(|patch| patch.target == RecordRef::Entity(id));
        let creation = member.posture == crate::mvcc::RequiredOutputRolePosture::Create;
        let wrote = writes(envelope, id, context)?;
        if creation {
            if envelope
                .record_allocations
                .iter()
                .filter(|allocation| allocation.record() == &RecordRef::Entity(id))
                .count()
                != 1
                || member.birth != envelope.commit.commit_id
                || member.birth_producer != membership.producer
                || member.birth_dependency != membership.dependency_identity
                || member.before_write.is_some()
                || patch
                    .is_none_or(|patch| patch.structural_change != RecordStructuralChange::Created)
                || member.after_write != envelope.commit.version_id
            {
                return Err("output lacks actual native birth allocation/patch".into());
            }
        } else if member.posture == crate::mvcc::RequiredOutputRolePosture::Retire {
            if member.after_write != envelope.commit.version_id
                || patch
                    .is_none_or(|patch| patch.structural_change != RecordStructuralChange::Deleted)
            {
                return Err("Retire differs from its actual native deletion".into());
            }
            continue;
        } else if wrote {
            if member.after_write != envelope.commit.version_id
                || patch
                    .is_some_and(|patch| patch.structural_change != RecordStructuralChange::Updated)
            {
                return Err("Preserve write differs from its actual native operation".into());
            }
        } else if member.before_write != Some(member.after_write) {
            return Err("unpatched Preserve member changed native write position".into());
        }
        if let Some(patch) = patch {
            if patch.contains_opaque_aspect {
                return Err("output has unclassified opaque cells".into());
            }
            let next = super::payload_reconstruction::apply(
                reconstructed
                    .as_ref()
                    .and_then(|state| state.state.as_ref()),
                &patch.authoritative_patch,
                plan,
                creation,
                lease,
                context,
            )?;
            if creation {
                envelope.validate_required_output_entity_create(
                    id,
                    plan,
                    next.state.as_ref(),
                    context,
                )?;
            }
            reconstructed = Some(next);
        }
    }
    Ok(reconstructed)
}

pub(super) fn writes(
    envelope: &CanonicalCommitEnvelope,
    id: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Stop> {
    charge(
        context,
        envelope
            .patch
            .authoritative_record_patches
            .len()
            .saturating_add(envelope.merged_plan.merged_intents.len()),
    )?;
    Ok(envelope
        .patch
        .authoritative_record_patches
        .iter()
        .any(|patch| patch.target == RecordRef::Entity(id))
        || envelope
            .merged_plan
            .merged_intents
            .iter()
            .any(|intent| intent_writes_entity(intent, id)))
}
