//! Reconstruct one live incident edge over its exact publication predecessors.
use super::{charge, memory_overflow, reserve, CodecRetirementClosureStop as Stop};
use crate::branch::RelationalBranchRoot;
use crate::history::data::{
    canonical_commit_envelope::CanonicalOutputPublication, CanonicalCommitEnvelope,
};
use crate::identity::data::RelationId;
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::RecordRef;
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

pub(super) fn validate(
    runtime: &RelationalRuntime,
    root: &RelationalBranchRoot,
    selected: &Arc<CanonicalCommitEnvelope>,
    id: RelationId,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let origin = selected.validate_output_membership_header()?;
    charge(context, origin.relations.len())?;
    let entry = origin
        .relations
        .iter()
        .find(|relation| relation.id == id)
        .ok_or("incident relation is outside complete publication membership")?;
    let partition = root
        .partition_state(id.partition_id)
        .ok_or("relation partition absent")?;
    let row = partition
        .relation_arena
        .get(&id)
        .filter(|row| row.is_live())
        .ok_or("relation generation absent or retired")?;
    let ends = row
        .extra()
        .endpoints
        .as_ref()
        .ok_or("relation endpoints absent")?;
    let write = partition
        .relation_arena
        .metadata_history_at(id.slot_index())
        .and_then(|rows| rows.get(rows.len().checked_sub(1)?))
        .ok_or("relation native write position absent")?
        .effective_at;
    if row.kind_id() != Some(entry.kind)
        || ends.source != entry.source
        || ends.target != entry.target
        || !super::materialization::write_matches(
            &RecordRef::Relation(id),
            selected.commit.commit_id,
            entry.after_write,
            write,
            &mut |commit| runtime.history.canonical_envelope(commit),
            &mut |version| runtime.history.canonical_envelope_for_version(version),
            runtime.history.catalog_len(),
            context,
        )?
    {
        return Err(
            "incident relation native position/endpoints differ from producer publication".into(),
        );
    }
    let plan = root
        .schema_authority()
        .relation_aspect_plan(entry.kind)
        .ok_or("relation physical contract plan absent")?;
    let state = reconstruct(runtime, selected, id, plan, context, lease)?;
    let payload = row.extra().authoritative_aspect_state.as_ref();
    charge(
        context,
        payload.map_or(0, |payload| payload.owned_allocation_capacity_bytes()),
    )?;
    if state.as_ref().and_then(|state| state.state.as_ref()) != payload {
        return Err("incident relation payload differs from native producer succession".into());
    }
    Ok(())
}

pub(super) fn reconstruct(
    runtime: &RelationalRuntime,
    selected: &Arc<CanonicalCommitEnvelope>,
    id: RelationId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Option<super::payload_reconstruction::ReconstructedPayload>, Stop> {
    let bound = runtime.history.catalog_len();
    let _memory = reserve(
        lease,
        bound
            .checked_mul(std::mem::size_of::<Arc<CanonicalCommitEnvelope>>())
            .ok_or_else(memory_overflow)?,
    )?;
    let mut chain = Vec::with_capacity(bound);
    let mut current = Arc::clone(selected);
    loop {
        if chain.len() >= bound {
            return Err("relation succession incomplete or cyclic".into());
        }
        let origin = current.validate_output_membership_header()?;
        charge(context, origin.relations.len())?;
        let relation = origin
            .relations
            .iter()
            .find(|relation| relation.id == id)
            .ok_or("relation predecessor membership absent")?;
        chain.push(Arc::clone(&current));
        let Some(before) = relation.before_write else {
            if relation.birth != current.commit.commit_id {
                return Err("relation birth differs".into());
            }
            break;
        };
        let CanonicalOutputPublication::Preserve { predecessor } = origin.publication else {
            return Err("relation has predecessor under initial publication".into());
        };
        let previous = runtime
            .history
            .canonical_envelope(predecessor)
            .ok_or("relation predecessor absent")?;
        let old = previous.validate_output_membership_header()?;
        charge(context, old.relations.len())?;
        let old = old
            .relations
            .iter()
            .find(|old| old.id == id)
            .ok_or("relation predecessor membership absent")?;
        if !super::materialization::write_matches(
            &RecordRef::Relation(id),
            previous.commit.commit_id,
            old.after_write,
            before,
            &mut |commit| runtime.history.canonical_envelope(commit),
            &mut |version| runtime.history.canonical_envelope_for_version(version),
            runtime.history.catalog_len(),
            context,
        )? || relation.birth != old.birth
            || relation.kind != old.kind
        {
            return Err("relation predecessor native coverage differs".into());
        }
        current = previous;
    }
    let mut state: Option<super::payload_reconstruction::ReconstructedPayload> = None;
    for envelope in chain.iter().rev() {
        let origin = envelope.validate_output_membership_header()?;
        charge(
            context,
            origin.relations.len() + envelope.patch.authoritative_record_patches.len(),
        )?;
        let relation = origin
            .relations
            .iter()
            .find(|relation| relation.id == id)
            .ok_or("relation membership absent")?;
        let patch = envelope
            .patch
            .authoritative_record_patches
            .iter()
            .find(|patch| patch.target == RecordRef::Relation(id));
        let creation = relation.before_write.is_none();
        if creation {
            charge(context, envelope.record_allocations.len())?;
            if envelope
                .record_allocations
                .iter()
                .filter(|allocation| allocation.record() == &RecordRef::Relation(id))
                .count()
                != 1
                || patch.is_none_or(|patch| patch.structural_change != Change::Created)
                || relation.after_write != envelope.commit.version_id
            {
                return Err("relation lacks actual native birth allocation/patch".into());
            }
        } else if super::publication_succession::relation_writes(envelope, id, context)? {
            if relation.after_write != envelope.commit.version_id
                || patch.is_some_and(|patch| patch.structural_change != Change::Updated)
            {
                return Err("relation mutation differs from actual native operation".into());
            }
        } else if relation.before_write != Some(relation.after_write) {
            return Err("unpatched relation changed native write position".into());
        }
        if let Some(patch) = patch {
            if patch.contains_opaque_aspect {
                return Err("incident relation contains unclassified cells".into());
            }
            state = Some(super::payload_reconstruction::apply(
                state.as_ref().and_then(|state| state.state.as_ref()),
                &patch.authoritative_patch,
                plan,
                creation,
                lease,
                context,
            )?);
            if creation {
                envelope.validate_required_output_relation_create(
                    id,
                    relation.kind,
                    state.as_ref().and_then(|state| state.state.as_ref()),
                    context,
                )?;
            }
        }
    }
    Ok(state)
}
