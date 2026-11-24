//! Canonical maintenance is an exact source/suspension/restoration transition.
use super::*;
mod source_ancestry;
use crate::history::data::canonical_commit_envelope::{
    OutputOriginHistory, OwnedValidatedOutputOrigins,
};
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::transactions::data::intents::materialization_intent::MaterializationMutationIntent as Materialization;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use source_ancestry::{source_gap, source_write};

pub(in super::super) fn validate(
    restored: &CanonicalCommitEnvelope,
    lookup: &mut impl FnMut(CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
    maximum: usize,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    let mut validated_history = OutputOriginHistory::default();
    validate_local(
        restored,
        lookup,
        maximum,
        &mut validated_history,
        execution,
        context,
    )?;
    let event = restored
        .required_output_materialization
        .as_ref()
        .ok_or("maintenance absent")?;
    let source = lookup(event.source_commit)?.ok_or("maintenance source absent")?;
    for publication in &event.producer_publications {
        let original = validated_history
            .by_commit(*publication, lookup, execution, context)?
            .ok_or("maintenance producer absent")?;
        for origin in original.output_origins() {
            charge(
                context,
                event
                    .records
                    .len()
                    .saturating_mul(origin.members.len() + origin.relations.len() + 1),
            )?;
            if event
                .records
                .iter()
                .any(|record| owns(origin, &record.record))
            {
                original.materialization_origin_for_route(&origin.meaning, context)?;
                source_gap(
                    &source,
                    &original,
                    origin,
                    lookup,
                    maximum,
                    &mut validated_history,
                    execution,
                    context,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_local(
    restored: &CanonicalCommitEnvelope,
    lookup: &mut impl FnMut(CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
    maximum: usize,
    validated_history: &mut OutputOriginHistory,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    let event = restored
        .required_output_materialization
        .as_ref()
        .ok_or("maintenance absent")?;
    super::super::require_linear_history(restored)?;
    if restored.required_output_origin.is_some()
        || !restored.required_output_codec_retirements.is_empty()
        || event.records.is_empty()
        || event.producer_publications.is_empty()
        || event
            .records
            .windows(2)
            .any(|pair| pair[0].record >= pair[1].record)
        || event
            .producer_publications
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || restored.commit.parents.as_slice() != [event.suspension_commit]
    {
        return Err("maintenance has mixed or incomplete canonical association".into());
    }
    let suspension = lookup(event.suspension_commit)?.ok_or("maintenance suspension absent")?;
    let source = lookup(event.source_commit)?.ok_or("maintenance source absent")?;
    if suspension.commit.parents.as_slice() != [event.source_commit]
        || suspension.branch_context != restored.branch_context
        || source.branch_context != restored.branch_context
        || source.commit.version_id >= suspension.commit.version_id
        || suspension.commit.version_id >= restored.commit.version_id
        || suspension.required_output_origin.is_some()
        || suspension.required_output_materialization.is_some()
    {
        return Err("maintenance source and suspension are not exact native predecessors".into());
    }
    let suspended_intents = &suspension.merged_plan.merged_intents;
    let restored_intents = &restored.merged_plan.merged_intents;
    charge(
        context,
        suspended_intents
            .len()
            .saturating_mul(restored_intents.len() + 1),
    )?;
    if suspended_intents.len() != restored_intents.len()
        || suspended_intents.iter().any(|intent| {
            record(intent, true).is_none_or(|id| {
                restored_intents
                    .iter()
                    .filter(|intent| record(intent, false) == Some(id.clone()))
                    .count()
                    != 1
            })
        })
        || restored_intents.iter().any(|intent| {
            record(intent, false).is_none_or(|id| {
                suspended_intents
                    .iter()
                    .filter(|intent| record(intent, true) == Some(id.clone()))
                    .count()
                    != 1
            })
        })
    {
        return Err("maintenance does not restore the exact native suspended manifest".into());
    }
    for member in &event.records {
        charge(
            context,
            1 + restored_intents.len() + restored.patch.authoritative_record_patches.len(),
        )?;
        if member.before_write > source.commit.version_id
            || source_write(&source, &member.record, lookup, maximum, context)?
                != member.before_write
            || !restored_intents
                .iter()
                .any(|intent| record(intent, false).as_ref() == Some(&member.record))
            || !restored
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| {
                    patch.target == member.record
                        && patch.structural_change == Change::Rematerialized
                })
            || !suspension
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| {
                    patch.target == member.record
                        && patch.structural_change == Change::MaterializationSuspended
                })
        {
            return Err("maintenance record lacks exact native suspend/restore operations".into());
        }
        let mut associations = 0;
        for publication in &event.producer_publications {
            charge(context, 1)?;
            let original = validated_history
                .by_commit(*publication, lookup, execution, context)?
                .ok_or("maintenance producer history absent")?;
            if original.branch_context != restored.branch_context {
                return Err("maintenance producer belongs to another branch".into());
            }
            if owns_record(&original, &member.record, context)? {
                associations += 1;
            }
        }
        if associations != 1 {
            return Err(
                "maintenance record lacks one complete original producer association".into(),
            );
        }
    }
    for publication in &event.producer_publications {
        let original = validated_history
            .by_commit(*publication, lookup, execution, context)?
            .ok_or("maintenance producer absent")?;
        let mut related = false;
        for record in &event.records {
            if owns_record(&original, &record.record, context)? {
                related = true;
                break;
            }
        }
        if !related {
            return Err("maintenance claims an unrelated complete producer".into());
        }
    }
    Ok(())
}

pub(super) fn owns(
    origin: &crate::history::data::canonical_commit_envelope::CanonicalRequiredOutputOrigin,
    record: &RecordRef,
) -> bool {
    match record {
        RecordRef::Entity(id) => origin.members.iter().any(|member| {
            member.id == *id && member.posture != crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire
        }),
        RecordRef::Relation(id) => origin.relations.iter().any(|relation| relation.id == *id),
    }
}

pub(super) fn record(intent: &MutationIntent, suspended: bool) -> Option<RecordRef> {
    match (intent, suspended) {
        (MutationIntent::Materialization(Materialization::SuspendEntity(value)), true) => {
            Some(RecordRef::Entity(value.entity_id))
        }
        (MutationIntent::Materialization(Materialization::SuspendRelation(value)), true) => {
            Some(RecordRef::Relation(value.relation_id))
        }
        (MutationIntent::Materialization(Materialization::RematerializeEntity(value)), false) => {
            Some(RecordRef::Entity(value.entity_id))
        }
        (MutationIntent::Materialization(Materialization::RematerializeRelation(value)), false) => {
            Some(RecordRef::Relation(value.relation_id))
        }
        _ => None,
    }
}

pub(super) fn owns_record(
    original: &OwnedValidatedOutputOrigins,
    record: &RecordRef,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Stop> {
    for origin in original.output_origins() {
        charge(context, origin.members.len() + origin.relations.len() + 1)?;
        if owns(origin, record) {
            original.materialization_origin_for_record(record, context)?;
            return Ok(true);
        }
    }
    Ok(false)
}
