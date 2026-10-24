//! Exact native creation position and canonical allocations classify relation custody.
use super::{charge, writes, CanonicalCommitEnvelope, CanonicalRequiredOutputOrigin, Stop};
use crate::identity::data::{EntityId, KindId, RelationId, VersionId};
use crate::publication::patch::data::RecordStructuralChange;
use crate::storage::overlay::{relation_partition_source_for_slot, PartitionAccess};
use crate::transactions::data::RecordRef;
use std::sync::Arc;
use worth_execution::MapKernelContext;

#[allow(clippy::too_many_arguments)]
pub(super) fn validate(
    state: &dyn PartitionAccess,
    id: RelationId,
    selected: &CanonicalRequiredOutputOrigin,
    current: Option<&CanonicalCommitEnvelope>,
    lookup: &mut impl FnMut(VersionId) -> Option<Arc<CanonicalCommitEnvelope>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    if ownership_at_creation(state, id, selected, current, lookup, context)? {
        return Err("producer-owned incident relation is omitted from selected succession".into());
    }
    Ok(())
}

/// Cold claims require ownership at the actual creation, not merely current incidence.
pub(in super::super) fn validate_owned(
    state: &dyn PartitionAccess,
    id: RelationId,
    selected: &CanonicalRequiredOutputOrigin,
    lookup: &mut impl FnMut(VersionId) -> Option<Arc<CanonicalCommitEnvelope>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    if !ownership_at_creation(state, id, selected, None, lookup, context)? {
        return Err("claimed relation was not producer-owned at birth".into());
    }
    Ok(())
}

fn ownership_at_creation(
    state: &dyn PartitionAccess,
    id: RelationId,
    selected: &CanonicalRequiredOutputOrigin,
    current: Option<&CanonicalCommitEnvelope>,
    lookup: &mut impl FnMut(VersionId) -> Option<Arc<CanonicalCommitEnvelope>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Stop> {
    charge(context, 3)?;
    let (_, partition) =
        relation_partition_source_for_slot(state, id.partition_id, id.slot_index())
            .ok_or("relation creation partition absent")?;
    let row = partition
        .relation_arena
        .get(&id)
        .filter(|row| row.is_live())
        .ok_or("relation creation generation absent")?;
    let version = partition
        .relation_arena
        .created_at_for_slot(id.slot_index())
        .ok_or("relation native creation position absent")?;
    let history = partition
        .relation_arena
        .metadata_history_at(id.slot_index())
        .ok_or("relation metadata history absent")?;
    let latest = history
        .get(
            history
                .len()
                .checked_sub(1)
                .ok_or("relation write position absent")?,
        )
        .ok_or("relation write metadata absent")?
        .effective_at;
    if latest != version {
        let held;
        charge(context, 1)?;
        let write = match current.filter(|e| e.commit.version_id == latest) {
            Some(envelope) => envelope,
            None => {
                held = lookup(latest).ok_or("relation latest canonical write absent")?;
                held.as_ref()
            }
        };
        if write.commit.version_id != latest || !writes(write, id, context)? {
            return Err("relation latest native write is unexplained".into());
        }
    }
    // Birth metadata may be trimmed; actual canonical Created endpoint operations
    // can supply those native facts. A membership claim never supplies its own proof.
    // Native metadata is indexed by version; never search canonical ancestry for a convenient create.
    let (mut low, mut high) = (0, history.len());
    while low < high {
        charge(context, 1)?;
        let middle = low + (high - low) / 2;
        if history
            .get(middle)
            .ok_or("relation metadata absent")?
            .effective_at
            < version
        {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    let birth = history
        .get(low)
        .filter(|m| m.effective_at == version && m.generation == id.generation_value())
        .map(|m| (m.kind_id, m.endpoints.source, m.endpoints.target));
    let held;
    charge(context, 1)?;
    let envelope = match current.filter(|e| e.commit.version_id == version) {
        Some(envelope) => envelope,
        None => {
            held = lookup(version).ok_or("relation exact canonical creation absent")?;
            held.as_ref()
        }
    };
    charge(
        context,
        envelope.record_allocations.len() + envelope.patch.authoritative_record_patches.len(),
    )?;
    if envelope.commit.version_id != version
        || envelope
            .record_allocations
            .iter()
            .filter(|a| a.record() == &RecordRef::Relation(id))
            .count()
            != 1
    {
        return Err("relation lacks unique canonical native allocation".into());
    }
    let mut patches = envelope
        .patch
        .authoritative_record_patches
        .iter()
        .filter(|p| p.target == RecordRef::Relation(id));
    let patch = patches
        .next()
        .ok_or("relation lacks exact canonical Created operation")?;
    if patch.structural_change != RecordStructuralChange::Created || patches.next().is_some() {
        return Err("relation lacks exact canonical Created operation".into());
    }
    let kind = row.kind_id().ok_or("relation kind absent")?;
    let birth = if envelope.required_output_origin.is_some() {
        actual_birth_endpoints(patch, birth, kind, context)?
    } else {
        birth
    };
    classify(envelope, selected, id, birth, kind, context)
}

#[allow(clippy::too_many_arguments)]
fn classify(
    envelope: &CanonicalCommitEnvelope,
    selected: &CanonicalRequiredOutputOrigin,
    id: RelationId,
    native_birth: Option<(KindId, EntityId, EntityId)>,
    kind: KindId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Stop> {
    let mut producer_owned = false;
    if envelope.required_output_origin.is_some() {
        let origin = envelope.required_output_origin.as_deref().unwrap();
        charge(
            context,
            origin.relations.len()
                + origin.outputs.len()
                + origin.members.len()
                + origin.meaning.family.len()
                + selected.meaning.family.len()
                + origin.birth_producer.len()
                + selected.birth_producer.len()
                + 2 * std::mem::size_of::<[u8; 32]>(),
        )?;
        let origin = envelope.validate_output_membership_header()?;
        let entry = origin.relations.iter().find(|entry| entry.id == id);
        let (_, source, target) =
            native_birth.ok_or("producer-side actual relation creation endpoints unavailable")?;
        let endpoints = match entry {
            Some(entry) => {
                if entry.before_write.is_some()
                    || entry.birth != envelope.commit.commit_id
                    || entry.after_write != envelope.commit.version_id
                    || entry.kind != kind
                    || (entry.kind, entry.source, entry.target) != (kind, source, target)
                {
                    return Err(
                        "relation birth roster differs from native allocation/endpoints".into(),
                    );
                }
                (source, target)
            }
            None => (source, target),
        };
        let touches = origin.outputs.binary_search(&endpoints.0).is_ok()
            || origin.outputs.binary_search(&endpoints.1).is_ok();
        if touches {
            entry.ok_or("producer-created incident relation is missing from its birth roster")?;
            producer_owned = origin.meaning.same_route(&selected.meaning)
                && origin.birth == selected.birth
                && origin.birth_producer == selected.birth_producer
                && origin.birth_dependency == selected.birth_dependency;
            let overlap_work = origin
                .outputs
                .len()
                .checked_mul(1 + (usize::BITS - selected.outputs.len().leading_zeros()) as usize)
                .ok_or(Stop::Request(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                ))?;
            charge(context, overlap_work)?;
            if !producer_owned
                && origin
                    .outputs
                    .iter()
                    .any(|id| selected.outputs.binary_search(id).is_ok())
            {
                return Err(
                    "foreign relation publication overlaps selected output ownership".into(),
                );
            }
        } else if entry.is_some() {
            return Err("relation birth claim has no producer output endpoint".into());
        }
    }
    Ok(producer_owned)
}

/// Read only the real Created patch's bound endpoints, never roster assertions.
fn actual_birth_endpoints(
    patch: &crate::publication::patch::data::PublishedAuthoritativeRecordPatch,
    native: Option<(KindId, EntityId, EntityId)>,
    kind: KindId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<(KindId, EntityId, EntityId)>, Stop> {
    use crate::publication::patch::data::{
        PublishedAuthoritativePatchOperation as Op, PublishedAuthoritativePatchValue as Value,
    };
    use worth_foundational::facade::{AspectBinding, AspectValue};
    if native.is_some_and(|birth| birth.0 != kind) {
        return Err("relation native birth kind differs".into());
    }
    let native = match patch.relation_birth {
        Some(birth) => {
            let structural = (birth.kind, birth.source, birth.target);
            if birth.kind != kind || native.is_some_and(|native| native != structural) {
                return Err(
                    "relation structural birth differs from native birth metadata/kind".into(),
                );
            }
            Some(structural)
        }
        None => native,
    };
    let (mut source, mut target) = match native {
        Some((_, source, target)) => (Some(source), Some(target)),
        None => (None, None),
    };
    let operations = patch.authoritative_patch.full_grammar_operations();
    charge(context, operations.len())?;
    for operation in operations {
        let (binding, value) = match operation {
            Op::WholeAspectSet { binding, value, .. } => (binding, Some(value)),
            Op::WholeAspectClear { binding, .. } | Op::FieldLevelPatch { binding, .. } => {
                (binding, None)
            }
        };
        let slot = match binding {
            AspectBinding::RelationSourceEndpoint => &mut source,
            AspectBinding::RelationTargetEndpoint => &mut target,
            _ => continue,
        };
        let Some(Value::Scalar(AspectValue::EntityRef(endpoint))) = value else {
            return Err("relation Created endpoint lacks an exact native reference".into());
        };
        let endpoint = EntityId::new(
            crate::identity::data::PartitionId(endpoint.partition_id.0),
            endpoint.local_slot.0,
            endpoint.generation.0,
        );
        if slot
            .replace(endpoint)
            .is_some_and(|previous| previous != endpoint)
        {
            return Err("relation Created endpoint differs from native birth metadata".into());
        }
    }
    Ok(source
        .zip(target)
        .map(|(source, target)| (kind, source, target)))
}

#[cfg(test)]
mod tests;
