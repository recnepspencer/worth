//! Entity and owned-relation maintenance must preserve canonical semantic payload.
use super::*;
use crate::history::data::CanonicalRequiredOutputMaterializationRecord;
use crate::publication::patch::data::RecordStructuralChange;
use crate::storage::overlay::{
    entity_partition_source_for_slot, relation_partition_source_for_slot, OverlayStateView,
    WorkingState,
};
use worth_execution::ExecutionResourceLease;

pub(super) fn candidate_matches(
    runtime: &crate::runtime::RelationalRuntime,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    state: &WorkingState,
    original: &Arc<CanonicalCommitEnvelope>,
    records: &[CanonicalRequiredOutputMaterializationRecord],
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<bool, Stop> {
    let origin = original.validate_output_membership_header()?;
    let after = OverlayStateView::new(basis.inner.root.as_ref(), state);
    for member in records {
        if !header::owns(origin, &member.record) {
            continue;
        }
        charge(context, 1)?;
        let matches = match member.record {
            RecordRef::Entity(id) => {
                let entry = origin
                    .members
                    .iter()
                    .find(|entry| entry.id == id)
                    .ok_or("maintenance entity association absent")?;
                let plan = basis
                    .inner
                    .root
                    .schema_authority()
                    .entity_aspect_plan(entry.kind)
                    .ok_or("maintenance entity contract absent")?;
                let expected = super::super::membership::reconstruct(
                    runtime, original, id, plan, context, lease,
                )?;
                let (_, partition) =
                    entity_partition_source_for_slot(&after, id.partition_id, id.slot_index())
                        .ok_or("maintenance entity partition absent")?;
                let row = partition
                    .entity_arena
                    .get(&id)
                    .filter(|row| row.is_live())
                    .ok_or("maintenance entity generation absent")?;
                if row.kind_id() != Some(entry.kind) {
                    return Ok(false);
                }
                let actual = row.extra().authoritative_aspect_state.as_ref();
                charge(
                    context,
                    actual.map_or(0, |state| state.owned_allocation_capacity_bytes()),
                )?;
                expected.as_ref().and_then(|state| state.state.as_ref()) == actual
            }
            RecordRef::Relation(id) => {
                let entry = origin
                    .relations
                    .iter()
                    .find(|entry| entry.id == id)
                    .ok_or("maintenance relation association absent")?;
                let plan = basis
                    .inner
                    .root
                    .schema_authority()
                    .relation_aspect_plan(entry.kind)
                    .ok_or("maintenance relation contract absent")?;
                let expected = super::super::relation_payload::reconstruct(
                    runtime, original, id, plan, context, lease,
                )?;
                let (_, partition) =
                    relation_partition_source_for_slot(&after, id.partition_id, id.slot_index())
                        .ok_or("maintenance relation partition absent")?;
                let row = partition
                    .relation_arena
                    .get(&id)
                    .filter(|row| row.is_live())
                    .ok_or("maintenance relation generation absent")?;
                if row.kind_id() != Some(entry.kind)
                    || !row.extra().endpoints.as_ref().is_some_and(|ends| {
                        ends.source == entry.source && ends.target == entry.target
                    })
                {
                    return Ok(false);
                }
                let actual = row.extra().authoritative_aspect_state.as_ref();
                charge(
                    context,
                    actual.map_or(0, |state| state.owned_allocation_capacity_bytes()),
                )?;
                expected.as_ref().and_then(|state| state.state.as_ref()) == actual
            }
        };
        if !matches {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn validate(
    runtime: &crate::runtime::RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    validate_one(runtime, envelope, context, lease)?;
    let event = envelope
        .required_output_materialization
        .as_ref()
        .ok_or("maintenance absent")?;
    let mut oldest = None;
    for id in &event.producer_publications {
        let version = runtime
            .history
            .canonical_envelope(*id)
            .ok_or("maintenance producer history absent")?
            .commit
            .version_id;
        oldest = Some(oldest.map_or(version, |old: VersionId| old.min(version)));
    }
    let oldest = oldest.ok_or("maintenance producers absent")?;
    let mut next = Some(event.source_commit);
    for _ in 0..runtime.history.catalog_len() {
        charge(context, 1)?;
        let previous = runtime
            .history
            .canonical_envelope(next.ok_or("maintenance source ancestry absent")?)
            .ok_or("maintenance source history absent")?;
        if previous.commit.version_id <= oldest {
            return Ok(());
        }
        if previous.required_output_materialization.is_some() {
            // The header pass has authenticated this pair and its source jump.
            // Validate its own payload once; do not recursively revisit its prefix.
            validate_one(runtime, &previous, context, lease)?;
        }
        next = previous
            .required_output_materialization
            .as_ref()
            .map(|event| event.source_commit)
            .or_else(|| previous.commit.parents.first().copied());
    }
    Err("maintenance semantic source ancestry is incomplete or cyclic".into())
}

fn validate_one(
    runtime: &crate::runtime::RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let event = envelope
        .required_output_materialization
        .as_ref()
        .ok_or("maintenance absent")?;
    for member in &event.records {
        let original = event
            .producer_publications
            .iter()
            .find_map(|id| {
                let envelope = runtime.history.canonical_envelope(*id)?;
                header::owns(envelope.required_output_origin.as_deref()?, &member.record)
                    .then_some(envelope)
            })
            .ok_or("maintenance producer association absent")?;
        let origin = original.validate_output_membership_header()?;
        let (plan, expected) = match member.record {
            RecordRef::Entity(id) => {
                let entry = origin
                    .members
                    .iter()
                    .find(|entry| entry.id == id)
                    .ok_or("maintenance entity absent")?;
                let plan = runtime
                    .schema_contract_runtime
                    .aspect_contract_plans
                    .entity_plans
                    .get(&entry.kind)
                    .ok_or("maintenance entity contract absent")?;
                if !envelope.merged_plan.merged_intents.iter().any(|intent| matches!(intent,
                    crate::transactions::data::MutationIntent::Materialization(crate::transactions::data::MaterializationMutationIntent::RematerializeEntity(value))
                    if value.entity_id == id && value.kind_id == entry.kind)) {
                    return Err("maintenance entity kind differs from producer".into());
                }
                (
                    plan,
                    super::super::membership::reconstruct_canonical_payload(
                        runtime, &original, id, plan, context, lease,
                    )?,
                )
            }
            RecordRef::Relation(id) => {
                let entry = origin
                    .relations
                    .iter()
                    .find(|entry| entry.id == id)
                    .ok_or("maintenance relation absent")?;
                let plan = runtime
                    .schema_contract_runtime
                    .aspect_contract_plans
                    .relation_plans
                    .get(&entry.kind)
                    .ok_or("maintenance relation contract absent")?;
                let intent_matches = envelope.merged_plan.merged_intents.iter().any(|intent| matches!(intent,
                    crate::transactions::data::MutationIntent::Materialization(crate::transactions::data::MaterializationMutationIntent::RematerializeRelation(value))
                    if value.relation_id == id && value.kind_id == entry.kind && value.source == entry.source && value.target == entry.target));
                if !intent_matches {
                    return Err("maintenance relation endpoints/kind differ from producer".into());
                }
                (
                    plan,
                    super::super::relation_payload::reconstruct(
                        runtime, &original, id, plan, context, lease,
                    )?,
                )
            }
        };
        let patch = envelope
            .patch
            .authoritative_record_patches
            .iter()
            .find(|patch| patch.target == member.record)
            .ok_or("maintenance payload patch absent")?;
        if patch.structural_change != RecordStructuralChange::Rematerialized
            || patch.contains_opaque_aspect
        {
            return Err("maintenance has an unsupported physical payload operation".into());
        }
        let actual = super::super::payload_reconstruction::apply(
            None,
            &patch.authoritative_patch,
            plan,
            true,
            lease,
            context,
        )?;
        if actual.state.as_ref() != expected.as_ref().and_then(|state| state.state.as_ref()) {
            return Err("maintenance changed canonical producer payload".into());
        }
    }
    Ok(())
}
