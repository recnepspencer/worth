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
    let approver = authenticated(&world, "bob", &request);
    let program = materialize_exact_approval(&world, &request, requested);

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
        let principal = authenticated(&world, subject, &request);
        let denial = approval_access(&world, &principal, &request)
            .err()
            .expect("actor policy must deny before lifecycle authority");
        assert_eq!(denial.kind(), expected);
        assert!(denial.identity().is_some());
        assert_eq!(denial.causes(), [expected]);
    }
}

#[test]
fn approval_after_the_exact_request_window_returns_the_request_receipt() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 12));
    let request = live_scope();
    let requested = super::request_support::commit_exact_request(&world, &request);
    super::request_support::resolve_exact_request_identities(&world, &request);
    world.authorization_time.script([time(106), time(106)]);
    let approver = authenticated(&world, "bob", &request);
    let access = approval_access(&world, &approver, &request).unwrap();
    let denial = world
        .application
        .authorize_elevation_approval(
            requested,
            access,
            &approval_operation(&world),
            Default::default(),
        )
        .err()
        .expect("approval after expiry must return the request receipt");
    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationExpired
    );
    assert_eq!(
        denial.into_requested().elevation_identity(),
        &string("elevation-2")
    );
}

#[test]
fn approval_command_cannot_select_a_different_elevation_subject() {
    let (world, request, requested) = requested_world(CapabilityElevationScenario::Active);
    let approver = authenticated(&world, "bob", &request);
    let access = approval_access_for_elevation(&world, &approver, &request, "elevation-1")
        .expect("the approver independently holds command authority for elevation-1");

    let denial = world
        .application
        .authorize_elevation_approval(
            requested,
            access,
            &approval_operation(&world),
            Default::default(),
        )
        .err()
        .expect("command authority for another subject must not consume this request");
    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationApprovalRejected
    );
}

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
    world: &World,
    subject: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Authenticated {
    let external = world.authenticate(subject, Duration::from_secs(60), request);
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub(super) fn approval_access(
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<Access, crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial>
{
    approval_access_for_elevation(world, principal, request, "elevation-2")
}

fn approval_access_for_elevation(
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
    world.selected_product().admit_capability_access(
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
    let program = materialize_exact_approval(world, request, requested);
    match world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(172, 172))
    {
        WorthQueryElevationApprovalOutcome::Approved(approved) => approved,
        unexpected => panic!("the canonical approval prerequisite must commit: {unexpected:?}"),
    }
}

pub(super) fn materialize_exact_approval(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requested: WorthQueryRequestedElevation,
) -> WorthQueryElevationApprovalProgram<
    IdentityExecutionSchema,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
    Account,
> {
    let approver = authenticated(world, "bob", request);
    let access = approval_access(world, &approver, request).unwrap();
    let operation = approval_operation(world);
    let admission = world
        .application
        .authorize_elevation_approval(requested, access, &operation, Default::default())
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| seal_approval_facts(reader))
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap()
        .materialize_elevation_approval_program()
        .unwrap()
}

fn string(value: &str) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::from(value),
    )
}
