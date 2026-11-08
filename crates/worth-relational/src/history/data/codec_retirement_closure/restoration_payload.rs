//! A restored birth must reproduce its exact original member, not merely valid bytes.
use super::{charge, CodecRetirementClosureStop as Stop};
use crate::{
    history::data::{CanonicalCommitEnvelope, CommitId},
    identity::data::EntityId,
    transactions::data::RecordRef,
};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_member(
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    current: &Arc<CanonicalCommitEnvelope>,
    id: EntityId,
    retirement: CommitId,
    predecessor: CommitId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let event = current
        .authored_cascade_restoration
        .as_ref()
        .ok_or("restored birth lost recorded inverse")?;
    let origin = current.output_origin_for_entity(id, context)?;
    charge(
        context,
        event.entities.len() + runtime.runtime.history.catalog_len(),
    )?;
    let old = event
        .entities
        .iter()
        .find(|(_, new)| *new == id)
        .ok_or("restored birth has no exact original member")?
        .0;
    let retired = runtime
        .commit(retirement)?
        .ok_or("original retirement unavailable")?;
    let original = runtime
        .commit(predecessor)?
        .ok_or("original producer unavailable")?;
    let prior = retired.output_origin_for_entity(old, context)?;
    let member = original
        .output_origin_for_entity(old, context)?
        .members
        .iter()
        .find(|member| member.id == old)
        .ok_or("original member unavailable")?;
    if event.retirement != retirement
        || prior.publication.predecessor() != Some(predecessor)
        || retired.commit.version_id >= current.commit.version_id
        || original.commit.version_id >= retired.commit.version_id
        || origin.producer != prior.producer
        || origin.dependency_identity != prior.dependency_identity
        || origin.birth != prior.birth
        || !origin.source_scope_owned
        || origin.meaning.implementation != prior.meaning.implementation
        || origin.meaning.family != prior.meaning.family
    {
        return Err("restoration changes original publication provenance".into());
    }
    let restored = origin
        .members
        .iter()
        .find(|member| member.id == id)
        .ok_or("restored member absent")?;
    if member.kind != restored.kind
        || member.role != restored.role
        || member.entity_name != restored.entity_name
    {
        return Err("restored member changes original role or kind".into());
    }
    let expected = super::membership::reconstruct(runtime, &original, old, plan, context, lease)?;
    charge(context, current.patch.authoritative_record_patches.len())?;
    let patch = current
        .patch
        .authoritative_record_patches
        .iter()
        .find(|patch| patch.target == RecordRef::Entity(id))
        .ok_or("restored birth has no actual patch")?;
    let actual = super::payload_reconstruction::apply(
        None,
        &patch.authoritative_patch,
        plan,
        true,
        lease,
        context,
    )?;
    if expected.as_ref().and_then(|p| p.state.as_ref()) != actual.state.as_ref() {
        return Err("restoration changed the original producer payload".into());
    }
    Ok(())
}

/// Compare the whole inverse roster to its exact recorded retirement before
/// any single member can be treated as a restored producer birth.
pub(super) fn validate_groups(
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    current: &CanonicalCommitEnvelope,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    let Some(event) = &current.authored_cascade_restoration else {
        return Ok(());
    };
    current.validate_restoration_header(context)?;
    charge(context, runtime.runtime.history.catalog_len())?;
    let retired = runtime
        .commit(event.retirement)?
        .ok_or("restoration retirement absent")?;
    let original = retired
        .authored_cascade_retirement
        .as_ref()
        .ok_or("restoration retirement is not authored")?;
    if original.anchor != event.original_anchor
        || original.publications.len() != event.publications.len()
        || retired.commit.version_id >= current.commit.version_id
    {
        return Err("restoration changes retired anchor or publication count".into());
    }
    let map = |id| {
        event
            .entities
            .iter()
            .find(|(old, _)| *old == id)
            .map_or(id, |(_, new)| *new)
    };
    for origin in &event.publications {
        let crate::history::data::CanonicalOutputPublication::AuthoredRestoration {
            predecessor,
            ..
        } = origin.publication
        else {
            return Err("restoration lost its predecessor".into());
        };
        charge(
            context,
            original
                .publications
                .len()
                .saturating_mul(origin.meaning.family.len() + event.entities.len() + 65),
        )?;
        let prior = original
            .publications
            .iter()
            .find(|prior| {
                prior.meaning.family == origin.meaning.family
                    && map(prior.meaning.source_scope) == origin.meaning.source_scope
                    && prior.meaning.source_partition == origin.meaning.source_partition
            })
            .ok_or("restoration invents a producer route")?;
        retired.output_origin_for_route(&prior.meaning, context)?;
        charge(
            context,
            origin.producer.len()
                + prior.producer.len()
                + origin.birth_producer.len()
                + prior.birth_producer.len()
                + 160,
        )?;
        if prior.publication.predecessor() != Some(predecessor)
            || prior.producer != origin.producer
            || prior.dependency_identity != origin.dependency_identity
            || prior.birth != origin.birth
            || prior.birth_producer != origin.birth_producer
            || prior.birth_dependency != origin.birth_dependency
            || prior.meaning.implementation != origin.meaning.implementation
            || prior.meaning.source_epoch != origin.meaning.source_epoch
            || prior.members.len() != origin.members.len()
        {
            return Err("restoration changes producer provenance or membership".into());
        }
        charge(context, runtime.runtime.history.catalog_len())?;
        let live = runtime
            .commit(predecessor)?
            .ok_or("restoration live predecessor absent")?;
        if live.commit.version_id >= retired.commit.version_id {
            return Err("restoration ancestry moves forward".into());
        }
        let live = live.output_origin_for_route(&prior.meaning, context)?;
        for member in live
            .members
            .iter()
            .filter(|member| member.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
        {
            charge(
                context,
                event.entities.len()
                    + origin
                        .members
                        .len()
                        .saturating_mul(member.role.len() + member.entity_name.len() + 8),
            )?;
            let id = map(member.id);
            if !origin.members.iter().any(|new| {
                new.id == id
                    && new.kind == member.kind
                    && new.role == member.role
                    && new.entity_name == member.entity_name
            }) {
                return Err("restoration changes original member correspondence".into());
            }
        }
        if live.relations.len() != origin.relations.len() {
            return Err("restoration omits original owned relations".into());
        }
        for edge in &live.relations {
            charge(
                context,
                event.relations.len() + event.entities.len() * 2 + origin.relations.len(),
            )?;
            let id = event
                .relations
                .iter()
                .find(|(old, _)| *old == edge.id)
                .ok_or("restoration omits relation mapping")?
                .1;
            if !origin.relations.iter().any(|new| {
                new.id == id
                    && new.kind == edge.kind
                    && new.source == map(edge.source)
                    && new.target == map(edge.target)
            }) {
                return Err("restoration changes owned relation correspondence".into());
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_relation(
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    current: &Arc<CanonicalCommitEnvelope>,
    id: crate::identity::data::RelationId,
    retirement: CommitId,
    predecessor: CommitId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    use worth_foundational::facade::*;
    validate_groups(runtime, current, context)?;
    let event = current
        .authored_cascade_restoration
        .as_ref()
        .ok_or("restored edge lost inverse")?;
    if event.retirement != retirement {
        return Err("restored edge changes retirement".into());
    }
    charge(
        context,
        event.relations.len() + runtime.runtime.history.catalog_len(),
    )?;
    let old = event
        .relations
        .iter()
        .find(|(_, new)| *new == id)
        .ok_or("restored edge lost original")?
        .0;
    let original = runtime
        .commit(predecessor)?
        .ok_or("restored edge predecessor absent")?;
    if original.commit.version_id >= current.commit.version_id {
        return Err("restored edge ancestry moves forward".into());
    }
    let expected =
        super::relation_payload::reconstruct(runtime, &original, old, plan, context, lease)?;
    let patch = super::authored_restoration::creation_patch(
        expected.as_ref().and_then(|p| p.state.as_ref()),
        plan,
        context,
    )?;
    let origin = current.output_origin_for_relation(id, context)?;
    charge(
        context,
        origin.relations.len() + current.patch.authoritative_record_patches.len(),
    )?;
    let edge = origin
        .relations
        .iter()
        .find(|edge| edge.id == id)
        .ok_or("restored edge absent")?;
    let candidate =
        crate::authority::mutation::intents::relation_endpoint_candidate::append_portable_in_scope(
            &patch,
            plan,
            edge.source,
            edge.target,
            lease,
            context,
        )
        .map_err(Stop::Request)?;
    candidate.compare(|patch| {
        let expected = match readmit_portable_record_aspect_patch(
            patch,
            PortablePatchReadmissionPurpose::RecordCreation,
            &|key: &AspectKey| plan.contract_for(key),
        ) {
            worth_proof::TransitionOutcome::Success(patch) => patch,
            worth_proof::TransitionOutcome::Denied(_) => {
                return Err("restored edge original contract differs".into())
            }
        };
        let expected = match expected.apply_to_optional(None) {
            worth_proof::TransitionOutcome::Success(state) => {
                state.map(|state| state.into_parts().into_parts().0)
            }
            worth_proof::TransitionOutcome::Denied(_) => {
                return Err("restored edge original payload differs".into())
            }
        };
        let actual = current
            .patch
            .authoritative_record_patches
            .iter()
            .find(|patch| patch.target == RecordRef::Relation(id))
            .ok_or("restored edge patch absent")?;
        let actual = super::payload_reconstruction::apply(
            None,
            &actual.authoritative_patch,
            plan,
            true,
            lease,
            context,
        )?;
        if expected.as_ref() != actual.state.as_ref() {
            return Err("restoration changed original edge payload".into());
        }
        Ok(())
    })
}
