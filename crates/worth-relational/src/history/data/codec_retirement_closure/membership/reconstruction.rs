//! Retain validated ancestry through canonical entity payload replay.
use super::super::reserve;
use super::{
    charge, writes, Arc, CanonicalCommitEnvelope, EntityId, ExecutionResourceLease,
    MapKernelContext, OutputOriginHistory, OwnedValidatedOutputOrigins, RecordRef,
    RecordStructuralChange, RelationalRuntime, Stop,
};
use crate::history::data::canonical_commit_envelope::CanonicalOutputPublication;
pub(in crate::history::data::codec_retirement_closure) fn reconstruct(
    runtime: &RelationalRuntime,
    selected: &OwnedValidatedOutputOrigins,
    id: EntityId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<Option<super::super::payload_reconstruction::ReconstructedPayload>, Stop> {
    reconstruct_chain(
        runtime,
        selected,
        id,
        plan,
        validated_history,
        context,
        lease,
        true,
        schema,
    )
}

/// Canonical producer patches supply expected bytes; their maintenance gaps
/// are validated by the enclosing semantic pass rather than recursively replayed.
pub(in crate::history::data::codec_retirement_closure) fn reconstruct_canonical_payload(
    runtime: &RelationalRuntime,
    selected: &OwnedValidatedOutputOrigins,
    id: EntityId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<Option<super::super::payload_reconstruction::ReconstructedPayload>, Stop> {
    reconstruct_chain(
        runtime,
        selected,
        id,
        plan,
        validated_history,
        context,
        lease,
        false,
        schema,
    )
}

#[allow(clippy::too_many_arguments)]
fn reconstruct_chain(
    runtime: &RelationalRuntime,
    selected: &OwnedValidatedOutputOrigins,
    id: EntityId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    validate_gap_payload: bool,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<Option<super::super::payload_reconstruction::ReconstructedPayload>, Stop> {
    super::super::restoration_payload::validate_groups(
        runtime,
        selected,
        validated_history,
        context,
        schema,
    )?;
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
            .ok_or_else(super::super::memory_overflow)?,
    )?;
    let mut chain = Vec::with_capacity(bound);
    let mut current = selected.clone();
    loop {
        if chain.len() >= bound {
            return Err("publication succession is cyclic or incomplete".into());
        }
        charge(context, bound)?;
        let current_origin = current.output_origin_for_route(&origin.meaning, context)?;
        chain.push(current.clone());
        match current_origin.publication {
            CanonicalOutputPublication::Birth
            | CanonicalOutputPublication::BirthAfterCodecRetirement { .. } => {
                if current_origin.birth != current.commit.commit_id
                    || current_origin.birth_producer != current_origin.producer
                    || current_origin.birth_dependency != current_origin.dependency_identity
                {
                    return Err("publication birth identity differs".into());
                }
                super::super::codec_disposition::validate_birth(
                    runtime, &current, context, lease, schema,
                )?;
                break;
            }
            CanonicalOutputPublication::AuthoredRestoration {
                retirement,
                predecessor,
            } => {
                super::super::restoration_payload::validate_groups(
                    runtime,
                    &current,
                    validated_history,
                    context,
                    schema,
                )?;
                // A later Preserve can introduce a distinct member after this
                // restoration base. Its actual Create is verified during replay;
                // only the base's members have an original restored payload.
                if current_origin
                    .members
                    .binary_search_by_key(&id, |member| member.id)
                    .is_ok()
                {
                    super::super::restoration_payload::validate_member(
                        runtime,
                        &current,
                        id,
                        retirement,
                        predecessor,
                        plan,
                        validated_history,
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
                let previous = validated_history
                    .by_commit(
                        predecessor,
                        &mut |commit| runtime.history.canonical_envelope(commit),
                        context,
                    )?
                    .ok_or("publication predecessor history missing")?;
                let prior = previous.output_origin_for_route(&current_origin.meaning, context)?;
                super::super::publication_succession::validate_succession(
                    runtime,
                    current.as_view(),
                    prior,
                    validated_history,
                    context,
                    schema,
                )?;
                super::super::publication_succession::validate_gap(
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
    let mut reconstructed: Option<super::super::payload_reconstruction::ReconstructedPayload> =
        None;
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
            let next = super::super::payload_reconstruction::apply(
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
