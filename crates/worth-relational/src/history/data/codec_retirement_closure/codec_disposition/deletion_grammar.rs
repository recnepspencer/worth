//! Exact logical deletion projection does not erase the retained physical payload.
use super::*;
use crate::schema::data::{AspectContractPlanCatalog, KindAspectContractDeclarations};
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(super) fn entity(
    grammar: &DeletionPublicationGrammar,
    envelope: &CanonicalCommitEnvelope,
    member: &CanonicalOutputMember,
    state: Option<&AuthoritativeRecordAspectState>,
    declarations: &KindAspectContractDeclarations,
    plans: &AspectContractPlanCatalog,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let actual = metadata::require_delete(envelope, RecordRef::Entity(member.id), context)?;
    let expected = (grammar.entity)(
        EntityDeletionExpectation {
            member,
            state,
            declarations,
            plans,
            actual,
        },
        context,
        lease,
    )?;
    compare(&actual.authoritative_patch, &expected.patch)
}

pub(super) fn relation(
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    envelope: &CanonicalCommitEnvelope,
    relation: &CanonicalRequiredOutputRelation,
    state: Option<&AuthoritativeRecordAspectState>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let actual = metadata::require_delete(envelope, RecordRef::Relation(relation.id), context)?;
    let expected = (runtime.input.deletion_grammar.relation)(
        RelationDeletionExpectation {
            relation,
            state,
            plans: runtime.input.plans,
            actual,
        },
        context,
        lease,
    )?;
    compare(&actual.authoritative_patch, &expected.patch)
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

/// One native deletion planner, supplied with the same stamped history inputs.
#[derive(Clone, Copy)]
pub(crate) struct DeletionPublicationGrammar {
    pub(crate) entity: fn(
        EntityDeletionExpectation<'_>,
        &mut MapKernelContext<'_, '_>,
        Option<&ExecutionResourceLease<'_>>,
    ) -> Result<FundedDeletionExpectation, Stop>,
    pub(crate) relation: fn(
        RelationDeletionExpectation<'_>,
        &mut MapKernelContext<'_, '_>,
        Option<&ExecutionResourceLease<'_>>,
    ) -> Result<FundedDeletionExpectation, Stop>,
}

pub(crate) struct FundedDeletionExpectation {
    pub(crate) patch: crate::publication::patch::data::PublishedAuthoritativePatch,
    pub(crate) _scratch: worth_execution::ExecutionMemoryReservation,
}

pub(crate) struct EntityDeletionExpectation<'a> {
    pub(crate) member: &'a CanonicalOutputMember,
    pub(crate) state: Option<&'a AuthoritativeRecordAspectState>,
    pub(crate) declarations: &'a KindAspectContractDeclarations,
    pub(crate) plans: &'a AspectContractPlanCatalog,
    pub(crate) actual: &'a crate::publication::patch::data::PublishedAuthoritativeRecordPatch,
}

pub(crate) struct RelationDeletionExpectation<'a> {
    pub(crate) relation: &'a CanonicalRequiredOutputRelation,
    pub(crate) state: Option<&'a AuthoritativeRecordAspectState>,
    pub(crate) plans: &'a AspectContractPlanCatalog,
    pub(crate) actual: &'a crate::publication::patch::data::PublishedAuthoritativeRecordPatch,
}
