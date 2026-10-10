use std::time::Duration;

use super::super::super::application_attempt::idempotency;
use super::super::super::fixture::{
    installed_elevated_capability_world, live_scope, Account, ApproveCapabilityElevationOperation,
    ApproveElevationCapability, ApproveElevationInput, CapabilityElevationApprover,
    CapabilityElevationGrant, CapabilityElevationIdentity, CapabilityElevationNotAfter,
    CapabilityElevationNotBefore, CapabilityElevationReason, CapabilityElevationRequester,
    CapabilityElevationResource, CapabilityElevationReview, CapabilityElevationScenario,
    CapabilityElevationStatusField, CapabilityReviewIdentity, CapabilityReviewKindField,
    CapabilityReviewResource, CapabilityReviewStatusField, CapabilityReviewer,
    IdentityExecutionSchema, Principal,
};
use super::super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryApprovedElevation,
    WorthQueryElevationApprovalOutcome, WorthQueryElevationApprovalProgram,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryPrincipalResolutionMode,
    WorthQueryRequestedElevation,
};

pub(super) type World = super::super::super::fixture::AuthorizationWorld;
pub(super) type Authenticated =
    crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        IdentityExecutionSchema,
        Principal,
        u64,
    >;
pub(super) type Access = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    ApproveElevationCapability,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
>;

#[test]
fn exact_request_approval_reobserves_lifecycle_and_commits_two_derived_effects() {
    let (world, request, requested) = requested_world(CapabilityElevationScenario::Active);
    let approver = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated(&host_custody, &world, "bob", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            materialize_exact_approval(&host_custody, &world, &request, requested)
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryElevationApprovalOutcome::Approved(approved) = world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(172, 172))
    else {
        panic!("the exact approval transition must commit");
    };
    assert_approved(&approved, &approver);
}

#[test]
fn requester_and_conflicted_approver_cannot_enter_approval_progression() {
    for (scenario, subject, expected) in [
        (
            CapabilityElevationScenario::Active,
            "alice",
            WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing,
        ),
        (
            CapabilityElevationScenario::ConflictedApprover,
            "bob",
            WorthQueryOperationAuthorizationDenialKind::ConflictRuleMatched,
        ),
    ] {
        let (world, request, _requested) = requested_world(scenario);
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                authenticated(&host_custody, &world, subject, &request)
            })
            .expect("the fixture read owns its installed host call");
        let denial = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                approval_access(&host_custody, &world, &principal, &request)
                    .err()
                    .expect("actor policy must deny before lifecycle authority")
            })
            .expect("the fixture read owns its installed host call");
        assert_eq!(denial.kind(), expected);
        assert!(denial.identity().is_some());
        assert_eq!(denial.causes(), [expected]);
    }
}

#[path = "approval_transition/approval_after_the_exact_request_window_returns_the_request_rece.rs"]
mod approval_after_the_exact_request_window_returns_the_request_rece;

#[path = "approval_transition/approval_command_cannot_select_a_different_elevation_subject.rs"]
mod approval_command_cannot_select_a_different_elevation_subject;

pub(super) fn requested_world(
    scenario: CapabilityElevationScenario,
) -> (
    World,
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    WorthQueryRequestedElevation,
) {
    requested_world_with_input(scenario, super::request_transition::honest_input())
}

pub(super) fn requested_world_with_input(
    scenario: CapabilityElevationScenario,
    input: super::super::super::fixture::RequestElevationInput,
) -> (
    World,
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    WorthQueryRequestedElevation,
) {
    let world = installed_elevated_capability_world(scenario);
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let requested = super::request_support::commit_request(&world, &request, input);
    super::request_support::resolve_exact_request_identities(&world, &request);
    (world, request, requested)
}

pub(super) fn authenticated(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    subject: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Authenticated {
    let borrowed_execution = phase_custody.borrowed_request_for_test();
    let execution = &borrowed_execution;

    let external = world.authenticate(subject, Duration::from_secs(60), request);
    world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &phase_custody.phase_for_application(&world.application),
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub(super) fn approval_access(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<Access, crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial>
{
    approval_access_for_elevation(
        &(phase_custody).test_host_request(),
        world,
        principal,
        request,
        "elevation-2",
    )
}

fn approval_access_for_elevation(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    elevation: &str,
) -> Result<Access, crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial>
{
    let capability = world
        .application
        .installed_schema()
        .capability(
            ApproveElevationCapability::reference(),
            ApproveCapabilityElevationOperation::reference(),
        )
        .unwrap();
    world
        .selected_product(phase_custody)
        .admit_capability_access(
            &phase_custody.phase_for_application(&world.application),
            principal,
            &capability,
            ApproveElevationInput {
                account: "account-1".to_owned(),
                elevation: elevation.to_owned(),
            },
            request,
        )
}

pub(super) fn approval_operation(
    world: &World,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
    IdentityExecutionSchema,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
> {
    world
        .application
        .installed_schema()
        .installed_operation(ApproveCapabilityElevationOperation::reference())
        .unwrap()
}

pub(super) fn seal_approval_facts(
    reader: &mut crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<
        IdentityExecutionSchema,
        ApproveCapabilityElevationOperation,
    >,
) {
    let elevation = reader
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            "elevation-2".to_owned(),
        )
        .unwrap();
    let review = reader
        .resolve_entity(CapabilityReviewIdentity::reference(), "review-2".to_owned())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationIdentity::reference())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationReason::reference())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationStatusField::reference())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationNotBefore::reference())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationNotAfter::reference())
        .unwrap();
    reader
        .require_decision_field(&review, CapabilityReviewIdentity::reference())
        .unwrap();
    reader
        .require_decision_field(&review, CapabilityReviewKindField::reference())
        .unwrap();
    reader
        .require_decision_field(&review, CapabilityReviewStatusField::reference())
        .unwrap();
    reader
        .decision_relations_to(CapabilityElevationRequester::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_to(CapabilityElevationApprover::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationGrant::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationResource::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationReview::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityReviewResource::reference(), &review)
        .unwrap();
    reader
        .decision_relations_to(CapabilityReviewer::reference(), &review)
        .unwrap();
}

fn assert_approved(approved: &WorthQueryApprovedElevation, approver: &Authenticated) {
    assert_eq!(approved.approval_changed_record_count(), 3);
    assert_eq!(approved.approval_emitted_effect_count(), 0);
    assert_eq!(approved.approver(), approver.principal_entity_id());
    assert_ne!(approved.requester(), approved.approver());
    assert!(approved.commits_share_branch());
}

pub(super) fn exact_approved_world() -> (
    World,
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    WorthQueryApprovedElevation,
) {
    let (world, request, requested) = requested_world(CapabilityElevationScenario::Active);
    let approved = approve_exact_request(&world, &request, requested);
    (world, request, approved)
}

pub(super) fn approve_exact_request(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requested: WorthQueryRequestedElevation,
) -> WorthQueryApprovedElevation {
    let program = world
        .application
        .with_application_advancement(request, |phase| {
            let phase_custody = phase.test_host_request();
            materialize_exact_approval(
                &(phase_custody).test_host_request(),
                world,
                request,
                requested,
            )
        })
        .expect("the approval materialization owns its host call");
    match world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(172, 172))
    {
        WorthQueryElevationApprovalOutcome::Approved(approved) => approved,
        unexpected => panic!("the canonical approval prerequisite must commit: {unexpected:?}"),
    }
}

pub(super) fn materialize_exact_approval(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requested: WorthQueryRequestedElevation,
) -> WorthQueryElevationApprovalProgram<
    IdentityExecutionSchema,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
    Account,
> {
    let approver = authenticated(&phase_custody, world, "bob", request);
    let access = approval_access(&phase_custody, world, &approver, request).unwrap();
    let operation = approval_operation(world);
    let admission = world
        .application
        .authorize_elevation_approval(
            &phase_custody.phase_for_application(&world.application),
            requested,
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
            |reader, _| seal_approval_facts(reader),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application(&world.application),
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            &phase_custody.phase_for_application(&world.application),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .materialize_elevation_approval_program()
        .unwrap()
}

fn string(value: &str) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::from(value),
    )
}
