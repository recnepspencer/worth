//! Canonical maintenance is an exact source/suspension/restoration transition.
use super::*;
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::transactions::data::{MaterializationMutationIntent as Materialization, MutationIntent};

pub(in super::super) fn validate(
    restored: &CanonicalCommitEnvelope,
    lookup: &mut impl FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
    maximum: usize,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    validate_local(restored, lookup, maximum, context)?;
    let event = restored
        .required_output_materialization
        .as_ref()
        .ok_or("maintenance absent")?;
    let source = lookup(event.source_commit).ok_or("maintenance source absent")?;
    for publication in &event.producer_publications {
        let original = lookup(*publication).ok_or("maintenance producer absent")?;
        source_gap(&source, &original, lookup, maximum, context)?;
    }
    Ok(())
}

fn validate_local(
    restored: &CanonicalCommitEnvelope,
    lookup: &mut impl FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
    maximum: usize,
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
    let suspension = lookup(event.suspension_commit).ok_or("maintenance suspension absent")?;
    let source = lookup(event.source_commit).ok_or("maintenance source absent")?;
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
            let original = lookup(*publication).ok_or("maintenance producer history absent")?;
            let origin = original.validate_output_membership_header()?;
            if original.branch_context != restored.branch_context {
                return Err("maintenance producer belongs to another branch".into());
            }
            if owns(origin, &member.record) {
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
        let original = lookup(*publication).ok_or("maintenance producer absent")?;
        let origin = original.validate_output_membership_header()?;
        if !event
            .records
            .iter()
            .any(|record| owns(origin, &record.record))
        {
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
            member.id == *id && member.posture != crate::mvcc::RequiredOutputRolePosture::Retire
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

fn source_gap(
    source: &CanonicalCommitEnvelope,
    original: &CanonicalCommitEnvelope,
    lookup: &mut impl FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
    maximum: usize,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    let origin = original.validate_output_membership_header()?;
    let mut next = Some(source.commit.commit_id);
    for _ in 0..maximum {
        charge(context, 1)?;
        let id = next.ok_or("maintenance original is outside source ancestry")?;
        if id == original.commit.commit_id {
            return Ok(());
        }
        let envelope = lookup(id).ok_or("maintenance source ancestry absent")?;
        if envelope.commit.version_id > source.commit.version_id {
            return Err("maintenance source ancestry moves forward".into());
        }
        super::super::require_linear_history(&envelope)?;
        if envelope
            .required_output_origin
            .as_ref()
            .is_some_and(|candidate| candidate.meaning.same_route(&origin.meaning))
            || envelope
                .required_output_codec_retirements
                .iter()
                .any(|candidate| candidate.meaning.same_route(&origin.meaning))
        {
            return Err("maintenance original producer was superseded at its source".into());
        }
        for record in origin.outputs.iter().copied().map(RecordRef::Entity).chain(
            origin
                .relations
                .iter()
                .map(|relation| RecordRef::Relation(relation.id)),
        ) {
            let wrote = match record {
                RecordRef::Entity(id) => super::super::membership::writes(&envelope, id, context)?,
                RecordRef::Relation(id) => {
                    super::super::publication_succession::relation_writes(&envelope, id, context)?
                }
            };
            if wrote {
                let event = envelope
                    .required_output_materialization
                    .as_ref()
                    .ok_or("ordinary native writer ended materialization producer continuity")?;
                if !event
                    .producer_publications
                    .contains(&original.commit.commit_id)
                    || !event.records.iter().any(|member| member.record == record)
                {
                    return Err("maintenance omitted a producer write".into());
                }
            }
        }
        if envelope.required_output_materialization.is_some() {
            validate_local(&envelope, lookup, maximum, context)?;
        }
        next = envelope
            .required_output_materialization
            .as_ref()
            .map(|event| event.source_commit)
            .or_else(|| envelope.commit.parents.first().copied());
    }
    Err("maintenance source ancestry is incomplete or cyclic".into())
}

fn source_write(
    source: &CanonicalCommitEnvelope,
    record: &RecordRef,
    lookup: &mut impl FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
    maximum: usize,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<VersionId, Stop> {
    let mut next = Some(source.commit.commit_id);
    for _ in 0..maximum {
        let envelope = lookup(next.ok_or("maintenance source write absent")?)
            .ok_or("maintenance source write history absent")?;
        let writes = match *record {
            RecordRef::Entity(id) => super::super::membership::writes(&envelope, id, context)?,
            RecordRef::Relation(id) => {
                super::super::publication_succession::relation_writes(&envelope, id, context)?
            }
        };
        if writes {
            return Ok(envelope.commit.version_id);
        }
        next = envelope.commit.parents.first().copied();
    }
    Err("maintenance source write ancestry incomplete or cyclic".into())
}
