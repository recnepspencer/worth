//! Re-admit governed complete codec retirement before it can end a route generation.
use super::{charge, request, CodecRetirementClosureStop as Stop};
use crate::capabilities::AspectPlanSource;
use crate::history::data::{
    canonical_commit_envelope::*, CanonicalCommitEnvelope, RequiredOutputRoutePosition,
};
use crate::runtime::RelationalRuntime;
use crate::transactions::data::{
    EntityMutationIntent, MutationIntent, RecordRef, RelationMutationIntent,
};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
mod activation;
mod deletion_grammar;
mod effect_coverage;
mod metadata;

pub(super) fn validate(
    runtime: &RelationalRuntime,
    envelope: &Arc<CanonicalCommitEnvelope>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    charge(
        context,
        envelope
            .required_output_codec_retirements
            .len()
            .saturating_add(1),
    )?;
    if envelope.required_output_codec_retirements.is_empty()
        || !envelope.codec_retirement_headers_valid()
    {
        return Err("invalid complete native codec disposition header".into());
    }
    let bytes = envelope
        .required_output_codec_retirements
        .len()
        .checked_mul(std::mem::size_of::<Arc<CanonicalRequiredOutputOrigin>>())
        .ok_or_else(super::memory_overflow)?;
    let _roster_memory = super::reserve(lease, bytes)?;
    let mut groups = Vec::with_capacity(envelope.required_output_codec_retirements.len());
    // Serialized sibling headers contribute no aggregate effect authority.
    for event in &envelope.required_output_codec_retirements {
        groups.push(validate_group(runtime, envelope, event, context, lease)?);
    }
    effect_coverage::validate(envelope, &groups, context)
}

fn validate_group(
    runtime: &RelationalRuntime,
    envelope: &Arc<CanonicalCommitEnvelope>,
    event: &CanonicalRequiredOutputCodecRetirement,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Arc<CanonicalRequiredOutputOrigin>, Stop> {
    let transition = envelope
        .schema_transition
        .as_ref()
        .ok_or("codec disposition has no actual schema transition")?;
    let source_commit = envelope
        .commit
        .parents
        .first()
        .copied()
        .ok_or("codec disposition has no exact source parent")?;
    if event.retiring != envelope.commit
        || envelope.required_output_origin.is_some()
        || envelope.commit.parents.len() != 1
        || event.source.branch_id() != &envelope.branch_context
        || envelope
            .branch_cell_checkpoint
            .as_ref()
            .map(|cell| &cell.observation)
            != Some(event.source.reference())
        || event
            .source
            .reference()
            .target()
            .as_basis()
            .map(|basis| basis.selected_commit_id())
            != Some(source_commit.0)
    {
        return Err("codec disposition differs from its exact retiring candidate/source".into());
    }
    super::require_linear_history(envelope)?;
    let source = runtime
        .history
        .canonical_envelope(source_commit)
        .ok_or("codec disposition source schema history absent")?;
    let original = runtime
        .history
        .canonical_envelope(event.publication)
        .ok_or("codec disposition original publication absent")?;
    let origin = original.validate_output_membership_header()?;
    charge(
        context,
        origin
            .meaning
            .family
            .len()
            .saturating_add(event.meaning.family.len())
            .saturating_add(96),
    )?;
    if origin.meaning != event.meaning {
        return Err("codec disposition names a foreign original publication".into());
    }
    super::publication_succession::validate_gap(
        runtime,
        envelope,
        event.publication,
        origin,
        context,
        lease,
        true,
    )?;
    let _metadata = metadata::reserve(&event.kinds, context, lease)?;
    let (before, after) =
        metadata::readmit_kinds(event, &source.schema_authority, &envelope.schema_authority)?;
    let changes = crate::runtime::codec_field_changes_in_context(&before, &after, context, lease)
        .map_err(|stop| match stop {
        crate::runtime::CodecFieldComparisonStop::Request(cause) => request(cause),
        crate::runtime::CodecFieldComparisonStop::Contract(detail) => Stop::Payload(detail),
    })?;
    if changes.is_empty() {
        return Err("codec disposition has no actual changed field meaning".into());
    }
    for change in changes.iter() {
        charge(
            context,
            transition.diff_atoms.len().saturating_mul(
                change
                    .aspect_key
                    .as_str()
                    .len()
                    .saturating_add(change.field_key.as_str().len())
                    .saturating_add(2),
            ),
        )?;
        let mut projection = change.aspect_key.as_str().to_owned();
        projection.push('.');
        projection.push_str(change.field_key.as_str());
        if !transition.diff_atoms.iter().any(|atom| atom.element.kind_id == Some(change.kind_id)
            && matches!(&atom.detail, crate::schema::data::SchemaDiffDetail::ProjectionContractChanged { projection_name } if projection_name.as_ref() == projection))
        { return Err("codec disposition differs from the canonical schema/codec transition".into()); }
    }
    let plans = crate::schema::lower_aspect_plans(&before);
    activation::validate(runtime, envelope, event, &plans, context, lease)?;
    for member in &origin.members {
        let plan = plans
            .entity_plans
            .get(&member.kind)
            .ok_or("codec disposition omits a complete member kind")?;
        let payload =
            super::membership::reconstruct(runtime, &original, member.id, plan, context, lease)?;
        if member.posture == crate::mvcc::RequiredOutputRolePosture::Retire {
            continue;
        }
        let present = payload.as_ref().and_then(|payload| payload.state.as_ref());
        charge(context, changes.len())?;
        if !changes.iter().any(|change| {
            change.kind_id == member.kind
                && present.is_some_and(|state| state.get(&change.aspect_key).is_some())
        }) {
            return Err("codec disposition retires a protected or omitted unchanged member".into());
        }
        deletion_grammar::entity(
            envelope,
            member,
            present,
            &before
                .entity_kinds
                .get(&member.kind)
                .ok_or("codec member source registration absent")?
                .aspect_contract_declarations,
            &plans,
            context,
            lease,
        )?;
    }
    for relation in &origin.relations {
        charge(
            context,
            source
                .schema_authority
                .relation_kinds
                .len()
                .saturating_add(envelope.schema_authority.relation_kinds.len()),
        )?;
        let before = source
            .schema_authority
            .relation_kinds
            .iter()
            .find(|kind| kind.kind_id == relation.kind)
            .ok_or("codec relation source contract absent")?;
        let after = envelope
            .schema_authority
            .relation_kinds
            .iter()
            .find(|kind| kind.kind_id == relation.kind)
            .ok_or("codec relation target contract absent")?;
        let plan = runtime
            .relation_aspect_plan(relation.kind)
            .ok_or("codec relation native contract absent")?;
        if before != after || plan.plan_revision != before.aspect_plan_revision {
            return Err("codec disposition changes relation meaning".into());
        }
        let payload = super::relation_payload::reconstruct(
            runtime,
            &original,
            relation.id,
            plan,
            context,
            lease,
        )?;
        deletion_grammar::relation(
            runtime,
            envelope,
            relation,
            payload.as_ref().and_then(|payload| payload.state.as_ref()),
            context,
            lease,
        )?;
    }
    Ok(Arc::clone(
        original.required_output_origin.as_ref().unwrap(),
    ))
}

pub(super) fn validate_birth(
    runtime: &RelationalRuntime,
    birth: &Arc<CanonicalCommitEnvelope>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let origin = birth.validate_output_membership_header()?;
    let prior = super::route_selection::select_at(
        runtime,
        birth.commit.parents.first().copied(),
        &origin.meaning,
        context,
        lease,
    )?;
    match (&origin.publication, prior) {
        (CanonicalOutputPublication::Birth, None) => Ok(()),
        (
            CanonicalOutputPublication::BirthAfterCodecRetirement { retirement },
            Some(RequiredOutputRoutePosition::CodecRetired(actual)),
        ) if *retirement == actual => Ok(()),
        _ => Err(
            "native Birth lacks exact never-published or governed codec-retired route basis".into(),
        ),
    }
}
