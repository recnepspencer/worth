//! Re-evaluate a retained codec deletion with the original mutation grammar.
use super::{entity_deletion_publication_patch, relation_deletion_publication_patch};
use crate::history::data::codec_retirement_closure::codec_disposition::deletion_grammar::{
    DeletionPublicationGrammar, EntityDeletionExpectation, FundedDeletionExpectation,
    RelationDeletionExpectation,
};
use crate::history::data::codec_retirement_closure::{
    charge, memory_overflow, reserve as reserve_execution, CodecRetirementClosureStop as Stop,
};
use crate::schema::aspect_plan_source::AspectPlanSource;

use crate::schema::data::LoweredAspectContractPlan;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(crate) const GRAMMAR: DeletionPublicationGrammar =
    DeletionPublicationGrammar { entity, relation };

fn entity(
    input: EntityDeletionExpectation<'_>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<FundedDeletionExpectation, Stop> {
    let plan = input
        .plans
        .entity_plans
        .get(&input.member.kind)
        .ok_or("codec member source plan absent")?;
    let scratch = reserve(plan, input.state, input.actual, context, lease)?;
    let patch = entity_deletion_publication_patch(
        input.member.id,
        input.member.kind,
        input.state,
        input.declarations,
        input.plans,
        input.actual.detail.clone(),
    )
    .map_err(|_| Stop::from("codec member deletion reconstruction refused"))?;
    Ok(FundedDeletionExpectation {
        patch,
        _scratch: scratch,
    })
}

fn relation(
    input: RelationDeletionExpectation<'_>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<FundedDeletionExpectation, Stop> {
    let plan = input
        .plans
        .relation_aspect_plan(input.relation.kind)
        .ok_or("codec relation plan absent")?;
    let scratch = reserve(plan, input.state, input.actual, context, lease)?;
    let patch = relation_deletion_publication_patch(
        input.relation.id,
        input.relation.kind,
        input.relation.source,
        input.relation.target,
        input.state,
        input.plans,
        input.actual.detail.clone(),
    )
    .map_err(|_| Stop::from("codec relation deletion reconstruction refused"))?;
    Ok(FundedDeletionExpectation {
        patch,
        _scratch: scratch,
    })
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
        .ok_or_else(memory_overflow)?;
    for binding in &plan.executable_bindings {
        charge(context, 1)?;
        bytes = bytes
            .checked_add(binding.contract.owned_allocation_capacity_bytes())
            .and_then(|n| {
                n.checked_add(
                    std::mem::size_of::<
                        crate::authority::mutation::canonical_deltas::data::EvaluatedAspectBinding,
                    >() + 256,
                )
            })
            .ok_or_else(memory_overflow)?;
    }
    // Deletion planner, evaluated evidence, foundational patch and portable
    // publication overlap; sixteen reached representations bound their clones.
    let bytes = bytes.checked_mul(16).ok_or_else(memory_overflow)?;
    charge(context, bytes)?;
    reserve_execution(lease, bytes)
}
