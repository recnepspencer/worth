//! Complete membership selection is independent of each member's latest write.
use super::{
    charge, intent_writes_entity, require_linear_history, CodecRetirementClosureStop as Stop,
};
use crate::history::data::canonical_commit_envelope::CanonicalRequiredOutputOrigin;
use crate::history::data::canonical_commit_envelope::{
    OutputOriginHistory, OwnedValidatedOutputOrigins,
};
use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;
use crate::{branch::AdmittedRelationalBranchBasis, identity::data::EntityId};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(super) fn latest(
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    id: EntityId,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<OwnedValidatedOutputOrigins, Stop> {
    let mut next = basis.inner.root.commit_id();
    let bound = runtime.runtime.history.catalog_len();
    for _ in 0..bound {
        charge(context, bound.saturating_add(1))?;
        let commit = next.ok_or("output correspondence has no complete native publication")?;
        let envelope = validated_history
            .by_commit(
                commit,
                &mut |commit| runtime.commit(commit),
                &runtime.execution(),
                context,
            )?
            .ok_or("output correspondence lost native history")?;
        require_linear_history(&envelope)?;
        for origin in envelope.output_origins() {
            charge(context, origin.outputs.len())?;
            if origin.outputs.binary_search(&id).is_ok() {
                envelope.output_origin_for_entity(id, context)?;
                return Ok(envelope.clone());
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
                    let original = validated_history
                        .by_commit(
                            *publication,
                            &mut |commit| runtime.commit(commit),
                            &runtime.execution(),
                            context,
                        )?
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
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    selected: &OwnedValidatedOutputOrigins,
    id: EntityId,
    payload: Option<&AuthoritativeRecordAspectState>,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Arc<CanonicalRequiredOutputOrigin>, Stop> {
    super::restoration_payload::validate_groups(
        runtime,
        selected,
        validated_history,
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
            &mut |commit| runtime.commit(commit),
            &mut |version, context| runtime.version(version, context),
            runtime.runtime.history.catalog_len(),
            &runtime.execution(),
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
        validated_history,
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

mod reconstruction;
pub(super) use reconstruction::{reconstruct, reconstruct_canonical_payload};

pub(super) fn writes(
    envelope: &crate::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody,
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
