use worth_query_declaration::facade::application_schema::OperationReads;

use super::super::super::application_attempt::idempotency;
use super::super::super::fixture::capability::{CapabilityElevation, CapabilityReview};
use super::super::super::fixture::{
    Account, CapabilityElevationApprover, CapabilityElevationGrant, CapabilityElevationIdentity,
    CapabilityElevationNotAfter, CapabilityElevationNotBefore, CapabilityElevationReason,
    CapabilityElevationRequester, CapabilityElevationResource, CapabilityElevationReview,
    CapabilityElevationStatusField, CapabilityReviewIdentity, CapabilityReviewKindField,
    CapabilityReviewResource, CapabilityReviewStatusField, CapabilityReviewer, CloseElevationInput,
    CompleteCapabilityReviewOperation, CompleteElevationReviewCapability,
    CompleteElevationReviewInput, IdentityExecutionSchema, RevokeCapabilityElevationOperation,
    RevokeElevationCapability,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApprovedElevation, WorthQueryCompleteApplicationReadSet,
    WorthQueryElevationCloseOutcome, WorthQueryElevationCloseProgram,
    WorthQueryInvariantEntityIdentity, WorthQueryMandatoryReview, WorthQueryMandatoryReviewOutcome,
    WorthQueryMandatoryReviewProgram, WorthQueryOperationAuthorizationDenial,
    WorthQueryProjectedApplicationMutation, WorthQueryReviewedElevation,
};

use super::approval_transition::{Authenticated, World};

type ElevationIdentity =
    WorthQueryInvariantEntityIdentity<IdentityExecutionSchema, CapabilityElevation>;
type ReviewIdentity = WorthQueryInvariantEntityIdentity<IdentityExecutionSchema, CapabilityReview>;

pub(super) type CloseReads = WorthQueryCompleteApplicationReadSet<
    IdentityExecutionSchema,
    RevokeCapabilityElevationOperation,
    CloseElevationInput,
    Account,
    WorthQueryProjectedApplicationMutation,
>;
pub(super) type ReviewReads = WorthQueryCompleteApplicationReadSet<
    IdentityExecutionSchema,
    CompleteCapabilityReviewOperation,
    CompleteElevationReviewInput,
    Account,
    WorthQueryProjectedApplicationMutation,
>;

pub(super) fn close_access(
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        RevokeElevationCapability,
        RevokeCapabilityElevationOperation,
        CloseElevationInput,
    >,
    WorthQueryOperationAuthorizationDenial,
> {
    close_access_with_input(world, principal, request, close_input())
}

pub(super) fn close_access_with_input(
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: CloseElevationInput,
) -> Result<
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        RevokeElevationCapability,
        RevokeCapabilityElevationOperation,
        CloseElevationInput,
    >,
    WorthQueryOperationAuthorizationDenial,
> {
    let capability = world
        .application
        .installed_schema()
        .capability(
            RevokeElevationCapability::reference(),
            RevokeCapabilityElevationOperation::reference(),
        )
        .unwrap();
    world
        .selected_product()
        .admit_capability_access(principal, &capability, input, request)
}

pub(super) fn review_access(
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        CompleteElevationReviewCapability,
        CompleteCapabilityReviewOperation,
        CompleteElevationReviewInput,
    >,
    WorthQueryOperationAuthorizationDenial,
> {
    review_access_with_input(world, principal, request, review_input())
}

pub(super) fn review_access_with_input(
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: CompleteElevationReviewInput,
) -> Result<
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        CompleteElevationReviewCapability,
        CompleteCapabilityReviewOperation,
        CompleteElevationReviewInput,
    >,
    WorthQueryOperationAuthorizationDenial,
> {
    let capability = world
        .application
        .installed_schema()
        .capability(
            CompleteElevationReviewCapability::reference(),
            CompleteCapabilityReviewOperation::reference(),
        )
        .unwrap();
    world
        .selected_product()
        .admit_capability_access(principal, &capability, input, request)
}

pub(super) fn materialize_close(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) -> WorthQueryElevationCloseProgram<
    IdentityExecutionSchema,
    RevokeCapabilityElevationOperation,
    CloseElevationInput,
    Account,
> {
    close_reads(world, request, approved)
        .materialize_elevation_close_program()
        .unwrap()
}

pub(super) fn close_reads(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) -> CloseReads {
    let closer = super::approval_transition::authenticated(world, "bob", request);
    let access = close_access(world, &closer, request).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(RevokeCapabilityElevationOperation::reference())
        .unwrap();
    let admission = world
        .application
        .authorize_elevation_close(approved, access, &operation, Default::default())
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| seal_lifecycle_facts(reader))
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap()
}

pub(super) fn close_exact(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) -> WorthQueryMandatoryReview {
    let program = materialize_close(world, request, approved);
    match world
        .application
        .compare_and_commit_elevation_close(program, idempotency(173, 173))
    {
        WorthQueryElevationCloseOutcome::Closed(mandatory) => mandatory,
        _ => panic!("the canonical close transition must commit"),
    }
}

pub(super) fn materialize_review(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    mandatory: WorthQueryMandatoryReview,
) -> WorthQueryMandatoryReviewProgram<
    IdentityExecutionSchema,
    CompleteCapabilityReviewOperation,
    CompleteElevationReviewInput,
    Account,
> {
    review_reads(world, request, mandatory)
        .materialize_mandatory_review_program()
        .unwrap()
}

pub(super) fn review_reads(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    mandatory: WorthQueryMandatoryReview,
) -> ReviewReads {
    let reviewer = super::approval_transition::authenticated(world, "carol", request);
    let access = review_access(world, &reviewer, request).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CompleteCapabilityReviewOperation::reference())
        .unwrap();
    let admission = world
        .application
        .authorize_mandatory_review(mandatory, access, &operation, Default::default())
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| seal_lifecycle_facts(reader))
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap()
}

pub(super) fn review_exact(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    mandatory: WorthQueryMandatoryReview,
) -> WorthQueryReviewedElevation {
    let program = materialize_review(world, request, mandatory);
    match world
        .application
        .compare_and_commit_mandatory_review(program, idempotency(174, 174))
    {
        WorthQueryMandatoryReviewOutcome::Reviewed(reviewed) => reviewed,
        _ => panic!("the canonical mandatory review must commit"),
    }
}

pub(super) fn seal_lifecycle_facts<Operation>(
    reader: &mut crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<
        IdentityExecutionSchema,
        Operation,
    >,
) where
    CapabilityElevationIdentity: OperationReads<Operation>,
    CapabilityElevationReason: OperationReads<Operation>,
    CapabilityElevationStatusField: OperationReads<Operation>,
    CapabilityElevationNotBefore: OperationReads<Operation>,
    CapabilityElevationNotAfter: OperationReads<Operation>,
    CapabilityReviewIdentity: OperationReads<Operation>,
    CapabilityReviewKindField: OperationReads<Operation>,
    CapabilityReviewStatusField: OperationReads<Operation>,
    CapabilityElevationRequester: OperationReads<Operation>,
    CapabilityElevationApprover: OperationReads<Operation>,
    CapabilityElevationGrant: OperationReads<Operation>,
    CapabilityElevationResource: OperationReads<Operation>,
    CapabilityElevationReview: OperationReads<Operation>,
    CapabilityReviewResource: OperationReads<Operation>,
    CapabilityReviewer: OperationReads<Operation>,
{
    let elevation = reader
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            "elevation-2".to_owned(),
        )
        .unwrap();
    let review = reader
        .resolve_entity(CapabilityReviewIdentity::reference(), "review-2".to_owned())
        .unwrap();
    seal_lifecycle_fields(reader, &elevation, &review);
    seal_lifecycle_relations(reader, &elevation, &review);
}

fn seal_lifecycle_fields<Operation>(
    reader: &mut crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<
        IdentityExecutionSchema,
        Operation,
    >,
    elevation: &ElevationIdentity,
    review: &ReviewIdentity,
) where
    CapabilityElevationIdentity: OperationReads<Operation>,
    CapabilityElevationReason: OperationReads<Operation>,
    CapabilityElevationStatusField: OperationReads<Operation>,
    CapabilityElevationNotBefore: OperationReads<Operation>,
    CapabilityElevationNotAfter: OperationReads<Operation>,
    CapabilityReviewIdentity: OperationReads<Operation>,
    CapabilityReviewKindField: OperationReads<Operation>,
    CapabilityReviewStatusField: OperationReads<Operation>,
{
    reader
        .require_decision_field(elevation, CapabilityElevationIdentity::reference())
        .unwrap();
    reader
        .require_decision_field(elevation, CapabilityElevationReason::reference())
        .unwrap();
    reader
        .require_decision_field(elevation, CapabilityElevationStatusField::reference())
        .unwrap();
    reader
        .require_decision_field(elevation, CapabilityElevationNotBefore::reference())
        .unwrap();
    reader
        .require_decision_field(elevation, CapabilityElevationNotAfter::reference())
        .unwrap();
    reader
        .require_decision_field(review, CapabilityReviewIdentity::reference())
        .unwrap();
    reader
        .require_decision_field(review, CapabilityReviewKindField::reference())
        .unwrap();
    reader
        .require_decision_field(review, CapabilityReviewStatusField::reference())
        .unwrap();
}

fn seal_lifecycle_relations<Operation>(
    reader: &mut crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<
        IdentityExecutionSchema,
        Operation,
    >,
    elevation: &ElevationIdentity,
    review: &ReviewIdentity,
) where
    CapabilityElevationRequester: OperationReads<Operation>,
    CapabilityElevationApprover: OperationReads<Operation>,
    CapabilityElevationGrant: OperationReads<Operation>,
    CapabilityElevationResource: OperationReads<Operation>,
    CapabilityElevationReview: OperationReads<Operation>,
    CapabilityReviewResource: OperationReads<Operation>,
    CapabilityReviewer: OperationReads<Operation>,
{
    reader
        .decision_relations_to(CapabilityElevationRequester::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_to(CapabilityElevationApprover::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationGrant::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationResource::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationReview::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityReviewResource::reference(), review)
        .unwrap();
    reader
        .decision_relations_to(CapabilityReviewer::reference(), review)
        .unwrap();
}

fn close_input() -> CloseElevationInput {
    CloseElevationInput {
        account: "account-1".to_owned(),
        elevation: "elevation-2".to_owned(),
    }
}

fn review_input() -> CompleteElevationReviewInput {
    CompleteElevationReviewInput {
        account: "account-1".to_owned(),
        elevation: "elevation-2".to_owned(),
        review: "review-2".to_owned(),
    }
}
