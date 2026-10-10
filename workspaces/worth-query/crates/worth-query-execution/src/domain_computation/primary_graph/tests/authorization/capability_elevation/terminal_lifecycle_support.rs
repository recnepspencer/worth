use crate::facade::runtime::ExecutionAllocationPolicy;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;
use worth_query_declaration::facade::application_schema::OperationReads;

use super::super::super::application_attempt::idempotency;
use super::super::super::fixture::capability::{
    CapabilityElevation, CapabilityElevationClosedAt, CapabilityReview, CapabilityReviewReviewedAt,
};
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
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
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
    close_access_with_input(
        &(phase_custody).test_host_request(),
        world,
        principal,
        request,
        close_input(),
    )
}

pub(super) fn close_access_with_input(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
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
        .selected_product(phase_custody)
        .admit_capability_access(
            &phase_custody.phase_for_application(&world.application),
            principal,
            &capability,
            input,
            request,
        )
}

pub(super) fn review_access(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
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
    review_access_with_input(
        &(phase_custody).test_host_request(),
        world,
        principal,
        request,
        review_input(),
    )
}

pub(super) fn review_access_with_input(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
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
        .selected_product(phase_custody)
        .admit_capability_access(
            &phase_custody.phase_for_application(&world.application),
            principal,
            &capability,
            input,
            request,
        )
}

pub(super) fn materialize_close(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) -> WorthQueryElevationCloseProgram<
    IdentityExecutionSchema,
    RevokeCapabilityElevationOperation,
    CloseElevationInput,
    Account,
> {
    close_reads(
        &(phase_custody).test_host_request(),
        world,
        request,
        approved,
    )
    .materialize_elevation_close_program()
    .unwrap()
}

#[path = "terminal_lifecycle_support/close_reads.rs"]
mod close_reads;
pub(super) use close_reads::close_reads;

pub(super) fn close_exact(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) -> WorthQueryMandatoryReview {
    let program = world
        .application
        .with_application_advancement(request, |phase| {
            let phase_custody = phase.test_host_request();
            materialize_close(
                &(phase_custody).test_host_request(),
                world,
                request,
                approved,
            )
        })
        .expect("the lifecycle materialization owns its host call");
    match world
        .application
        .compare_and_commit_elevation_close(program, idempotency(173, 173))
    {
        WorthQueryElevationCloseOutcome::Closed(mandatory) => mandatory,
        _ => panic!("the canonical close transition must commit"),
    }
}

pub(super) fn materialize_review(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    mandatory: WorthQueryMandatoryReview,
) -> WorthQueryMandatoryReviewProgram<
    IdentityExecutionSchema,
    CompleteCapabilityReviewOperation,
    CompleteElevationReviewInput,
    Account,
> {
    review_reads(
        &(phase_custody).test_host_request(),
        world,
        request,
        mandatory,
    )
    .materialize_mandatory_review_program()
    .unwrap()
}

pub(super) fn review_reads(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    mandatory: WorthQueryMandatoryReview,
) -> ReviewReads {
    let reviewer =
        super::approval_transition::authenticated(&phase_custody, world, "carol", request);
    let access = review_access(&phase_custody, world, &reviewer, request).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CompleteCapabilityReviewOperation::reference())
        .unwrap();
    let admission = world
        .application
        .authorize_mandatory_review(
            &phase_custody.phase_for_application(&world.application),
            mandatory,
            access,
            &operation,
            Default::default(),
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &phase_custody.phase_for_application(&world.application),
            &admission,
            |reader, _| {
                seal_lifecycle_facts(reader);
                let review = reader
                    .resolve_entity(CapabilityReviewIdentity::reference(), "review-2".to_owned())
                    .unwrap();
                reader
                    .require_decision_field(&review, CapabilityReviewReviewedAt::reference())
                    .unwrap();
            },
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application(&world.application),
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            &phase_custody.phase_for_application(&world.application),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}

pub(super) fn review_exact(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    mandatory: WorthQueryMandatoryReview,
) -> WorthQueryReviewedElevation {
    let program = world
        .application
        .with_application_advancement(request, |phase| {
            let phase_custody = phase.test_host_request();
            materialize_review(&phase_custody, world, request, mandatory)
        })
        .expect("the lifecycle materialization owns its host call");
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

#[path = "terminal_lifecycle_support/lifecycle_facts.rs"]
mod lifecycle_facts;
use lifecycle_facts::{seal_lifecycle_fields, seal_lifecycle_relations};
