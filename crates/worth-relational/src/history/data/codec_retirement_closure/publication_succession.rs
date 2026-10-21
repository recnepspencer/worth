//! Exact complete publication edges, including dynamic births and retirements.
use super::membership::writes;
use super::require_linear_history;
use super::{charge, CodecRetirementClosureStop as Stop};
use crate::history::data::canonical_commit_envelope::CanonicalRequiredOutputOrigin;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::publication::patch::data::RecordStructuralChange;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::RecordRef;
use worth_execution::MapKernelContext;

pub(super) fn validate_succession(
    envelope: &CanonicalCommitEnvelope,
    prior: &CanonicalRequiredOutputOrigin,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    let current = envelope.validate_output_membership_header()?;
    charge(
        context,
        current.members.len().saturating_add(prior.members.len()),
    )?;
    if current.birth != prior.birth
        || current.birth_producer != prior.birth_producer
        || current.birth_dependency != prior.birth_dependency
        || current.meaning.family != prior.meaning.family
        || current.meaning.source_scope != prior.meaning.source_scope
        || current.meaning.source_partition != prior.meaning.source_partition
    {
        return Err("publication succession changes complete family/source membership".into());
    }
    use crate::mvcc::RequiredOutputRolePosture as Posture;
    for member in &current.members {
        charge(
            context,
            (usize::BITS - prior.members.len().leading_zeros()) as usize
                + member.birth_producer.len()
                + 1,
        )?;
        let old = prior
            .members
            .binary_search_by_key(&member.id, |old| old.id)
            .ok()
            .map(|index| &prior.members[index]);
        if member.posture == Posture::Create {
            if old.is_some()
                || member.birth != current.commit.commit_id
                || member.before_write.is_some()
                || member.birth_producer != current.producer
                || member.birth_dependency != current.dependency_identity
            {
                return Err("new member lacks this publication's distinct birth".into());
            }
        } else {
            let old = old
                .filter(|old| old.posture != Posture::Retire)
                .ok_or("successor revives or invents prior live membership")?;
            if member.kind != old.kind
                || member.entity_name != old.entity_name
                || member.before_write != Some(old.after_write)
                || member.birth != old.birth
                || member.birth_producer != old.birth_producer
                || member.birth_dependency != old.birth_dependency
            {
                return Err("publication predecessor member/write correspondence differs".into());
            }
        }
    }
    for old in &prior.members {
        charge(
            context,
            (usize::BITS - current.members.len().leading_zeros()) as usize + 1,
        )?;
        if old.posture != Posture::Retire
            && current
                .members
                .binary_search_by_key(&old.id, |member| member.id)
                .is_err()
        {
            return Err("successor omits prior live membership".into());
        }
    }
    for relation in &current.relations {
        charge(context, prior.relations.len() + 1)?;
        let old = prior.relations.iter().find(|old| old.id == relation.id);
        if let Some(before) = relation.before_write {
            let old = old.ok_or("relation invents predecessor membership")?;
            if before != old.after_write || relation.birth != old.birth || relation.kind != old.kind
            {
                return Err("relation predecessor native position differs".into());
            }
        } else if old.is_some() || relation.birth != current.commit.commit_id {
            return Err("new relation lacks this publication's distinct birth".into());
        }
    }
    for old in &prior.relations {
        charge(
            context,
            current.relations.len() + envelope.patch.authoritative_record_patches.len() + 1,
        )?;
        if !current
            .relations
            .iter()
            .any(|relation| relation.id == old.id)
            && !envelope
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| {
                    patch.target == RecordRef::Relation(old.id)
                        && matches!(
                            patch.structural_change,
                            RecordStructuralChange::Deleted
                                | RecordStructuralChange::RetainedForAudit
                        )
                })
        {
            return Err("successor omits prior relation without actual native retirement".into());
        }
    }
    Ok(())
}

pub(super) fn validate_gap(
    runtime: &RelationalRuntime,
    current: &CanonicalCommitEnvelope,
    predecessor: CommitId,
    origin: &CanonicalRequiredOutputOrigin,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    let mut next = current.commit.parents.first().copied();
    let bound = runtime.history.catalog_len();
    for _ in 0..bound {
        charge(context, bound)?;
        let commit = next.ok_or("publication predecessor is outside canonical ancestry")?;
        if commit == predecessor {
            return Ok(());
        }
        let envelope = runtime
            .history
            .canonical_envelope(commit)
            .ok_or("publication gap lost native history")?;
        require_linear_history(&envelope)?;
        if let Some(publication) = &envelope.required_output_origin {
            charge(
                context,
                publication.meaning.family.len() + origin.meaning.family.len() + 64,
            )?;
            if publication.meaning.same_route(&origin.meaning) {
                return Err(
                    "intervening complete family publication breaks exact predecessor succession"
                        .into(),
                );
            }
        }

        for event in &envelope.required_output_codec_retirements {
            charge(
                context,
                event
                    .meaning
                    .family
                    .len()
                    .saturating_add(origin.meaning.family.len())
                    .saturating_add(64),
            )?;
            if event.meaning.same_route(&origin.meaning) {
                return Err(
                    "intervening governed codec disposition ended producer continuity".into(),
                );
            }
        }
        for member in &origin.members {
            if writes(&envelope, member.id, context)?
                || envelope
                    .required_output_origin
                    .as_ref()
                    .is_some_and(|publication| {
                        publication.outputs.binary_search(&member.id).is_ok()
                    })
            {
                return Err(
                    "intervening write or publication breaks exact predecessor succession".into(),
                );
            }
        }
        for relation in &origin.relations {
            if relation_writes(&envelope, relation.id, context)? {
                return Err("intervening relation write breaks producer continuity".into());
            }
        }
        next = envelope.commit.parents.first().copied();
    }
    Err("publication predecessor gap is incomplete or cyclic".into())
}

pub(super) use crate::history::data::publication_relations::writes as relation_writes;
