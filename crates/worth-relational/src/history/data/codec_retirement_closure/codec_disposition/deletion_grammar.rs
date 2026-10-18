//! Exact logical deletion projection does not erase the retained physical payload.
use super::*;
use crate::schema::data::{
    AspectContractPlanCatalog, KindAspectContractDeclarations, LoweredAspectContractPlan,
};
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(super) fn entity(
    envelope: &CanonicalCommitEnvelope,
    member: &CanonicalOutputMember,
    state: Option<&AuthoritativeRecordAspectState>,
    declarations: &KindAspectContractDeclarations,
    plans: &AspectContractPlanCatalog,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let actual = metadata::require_delete(envelope, RecordRef::Entity(member.id), context)?;
    let plan = plans
        .entity_plans
        .get(&member.kind)
        .ok_or("codec member source plan absent")?;
    let _scratch = reserve(plan, state, actual, context, lease)?;
    let expected = crate::authority::mutation::entity_deletion_publication_patch(
        member.id,
        member.kind,
        state,
        declarations,
        plans,
        actual.detail.clone(),
    )
    .map_err(|_| Stop::from("codec member deletion reconstruction refused"))?;
    compare(&actual.authoritative_patch, &expected)
}

pub(super) fn relation(
    runtime: &RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    relation: &CanonicalRequiredOutputRelation,
    state: Option<&AuthoritativeRecordAspectState>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let actual = metadata::require_delete(envelope, RecordRef::Relation(relation.id), context)?;
    let plan = runtime
        .relation_aspect_plan(relation.kind)
        .ok_or("codec relation plan absent")?;
    let _scratch = reserve(plan, state, actual, context, lease)?;
    let expected = crate::authority::mutation::relation_deletion_publication_patch(
        relation.id,
        relation.kind,
        relation.source,
        relation.target,
        state,
        runtime,
        actual.detail.clone(),
    )
    .map_err(|_| Stop::from("codec relation deletion reconstruction refused"))?;
    compare(&actual.authoritative_patch, &expected)
}

fn compare(
    actual: &crate::publication::patch::data::PublishedAuthoritativePatch,
    expected: &crate::publication::patch::data::PublishedAuthoritativePatch,
) -> Result<(), Stop> {
    if actual != expected {
        return Err("codec disposition differs from exact native deletion grammar".into());
    }
    Ok(())
}

fn reserve(
    plan: &LoweredAspectContractPlan,
    state: Option<&AuthoritativeRecordAspectState>,
    patch: &crate::publication::patch::data::PublishedAuthoritativeRecordPatch,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<worth_execution::ExecutionMemoryReservation, Stop> {
    let mut bytes = state
        .map_or(0, |state| state.owned_allocation_capacity_bytes())
        .checked_add(patch.authoritative_patch.owned_allocation_capacity_bytes() as usize)
        .and_then(|n| n.checked_add(patch.detail.owned_allocation_capacity_bytes() as usize))
        .ok_or_else(super::super::memory_overflow)?;
    for binding in &plan.executable_bindings {
        charge(context, 1)?;
        bytes = bytes
            .checked_add(binding.contract.owned_allocation_capacity_bytes())
            .and_then(|n| {
                n.checked_add(
                    std::mem::size_of::<crate::authority::mutation::EvaluatedAspectBinding>() + 256,
                )
            })
            .ok_or_else(super::super::memory_overflow)?;
    }
    // Deletion planner, evaluated evidence, foundational patch and portable
    // publication overlap; sixteen reached representations bound their clones.
    let bytes = bytes
        .checked_mul(16)
        .ok_or_else(super::super::memory_overflow)?;
    charge(context, bytes)?;
    super::super::reserve(lease, bytes)
}
