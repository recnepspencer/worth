//! Reconstruct the protected program row through the exact retiring update.
use super::*;
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::schema::data::AspectContractPlanCatalog;

pub(super) fn validate(
    runtime: &RelationalRuntime,
    envelope: &Arc<CanonicalCommitEnvelope>,
    event: &CanonicalRequiredOutputCodecRetirement,
    plans: &AspectContractPlanCatalog,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let id = event.activation.entity_id;
    let root = runtime
        .history
        .branch_cell(&envelope.branch_context)
        .and_then(|cell| cell.root())
        .ok_or("codec program native branch root absent")?;
    charge(context, root.entity_slot_count().saturating_add(1))?;
    // Kind is immutable for this exact native generation. Its value at the
    // retirement is reconstructed below, independently of today's program.
    let kind = root
        .partition_state(id.partition_id)
        .and_then(|partition| partition.entity_arena.get(&id))
        .and_then(|row| row.kind_id())
        .ok_or("codec program native generation absent")?;
    let plan = plans
        .entity_plans
        .get(&kind)
        .ok_or("codec disposition omits the actual protected program contract")?;
    let bound = runtime.history.catalog_len();
    let _memory = super::super::reserve(
        lease,
        bound
            .checked_mul(std::mem::size_of::<Arc<CanonicalCommitEnvelope>>())
            .ok_or_else(super::super::memory_overflow)?,
    )?;
    let mut chain = Vec::with_capacity(bound);
    let mut next = envelope.commit.parents.first().copied();
    let mut born = false;
    for _ in 0..bound {
        let commit = next.ok_or("protected program history has no native allocation")?;
        charge(context, bound.saturating_add(1))?;
        let current = runtime
            .history
            .canonical_envelope(commit)
            .ok_or("protected program history absent")?;
        super::super::require_linear_history(&current)?;
        charge(context, current.patch.authoritative_record_patches.len())?;
        if let Some(patch) = current
            .patch
            .authoritative_record_patches
            .iter()
            .find(|patch| patch.target == RecordRef::Entity(id))
        {
            if !matches!(patch.structural_change, Change::Created | Change::Updated)
                || patch.contains_opaque_aspect
            {
                return Err(
                    "protected program has an unsupported retirement or opaque write".into(),
                );
            }
            let creation = patch.structural_change == Change::Created;
            if creation {
                charge(context, current.record_allocations.len())?;
                if current
                    .record_allocations
                    .iter()
                    .filter(|allocation| allocation.record() == &RecordRef::Entity(id))
                    .count()
                    != 1
                {
                    return Err("protected program has no exact native allocation".into());
                }
            }
            chain.push(Arc::clone(&current));
            if creation {
                born = true;
                break;
            }
        } else if super::super::membership::writes(&current, id, context)? {
            return Err("protected program has an unclassified native write".into());
        }
        next = current.commit.parents.first().copied();
    }
    if !born {
        return Err("protected program ancestry is incomplete or cyclic".into());
    }
    let mut state: Option<super::super::payload_reconstruction::ReconstructedPayload> = None;
    for current in chain.iter().rev() {
        let patch = current
            .patch
            .authoritative_record_patches
            .iter()
            .find(|patch| patch.target == RecordRef::Entity(id))
            .ok_or("protected program patch absent")?;
        state = Some(super::super::payload_reconstruction::apply(
            state.as_ref().and_then(|state| state.state.as_ref()),
            &patch.authoritative_patch,
            plan,
            patch.structural_change == Change::Created,
            lease,
            context,
        )?);
    }
    let patch = envelope
        .patch
        .authoritative_record_patches
        .iter()
        .find(|patch| patch.target == RecordRef::Entity(id))
        .ok_or("codec program activation patch absent")?;
    if patch.structural_change != Change::Updated || patch.contains_opaque_aspect {
        return Err("codec program activation is not its exact native update".into());
    }
    let after = super::super::payload_reconstruction::apply(
        state.as_ref().and_then(|state| state.state.as_ref()),
        &patch.authoritative_patch,
        plan,
        false,
        lease,
        context,
    )?;
    crate::history::data::codec_retirement_preservation::compare_activation_payload(
        state.as_ref().and_then(|state| state.state.as_ref()),
        after.state.as_ref(),
        &event.activation,
    )
    .map_err(Stop::from)
}
