use std::time::Duration;

use worth_foundational::facade::{AspectValue, InternedString};

use super::super::super::application_attempt::{authenticated_principal, idempotency};
use super::super::super::fixture::{
    installed_elevated_capability_world, live_scope, Account, AccountLabel, CapabilityAction,
    CapabilityDisclosure, CapabilityElevationIdentity, CapabilityElevationScenario,
    CapabilityPurpose, CapabilityReviewIdentity, IdentityExecutionSchema,
    RequestCapabilityElevationOperation, RequestElevationCapability, RequestElevationInput,
};
use super::super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitOutcome, WorthQueryCompleteApplicationReadSet,
    WorthQueryElevationRequestOutcome, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryPrincipalResolutionMode,
    WorthQueryProjectedApplicationMutation,
};

type World = super::super::super::fixture::AuthorizationWorld;
type Principal = super::super::super::fixture::Principal;
type Access = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    RequestElevationCapability,
    RequestCapabilityElevationOperation,
    RequestElevationInput,
>;
type Reads = WorthQueryCompleteApplicationReadSet<
    IdentityExecutionSchema,
    RequestCapabilityElevationOperation,
    RequestElevationInput,
    Account,
    WorthQueryProjectedApplicationMutation,
>;

#[test]
fn exact_request_commits_query_derived_state_and_returns_one_requested_receipt() {
    let world = request_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let program = request_reads(&world, &principal, &request, honest_input())
        .materialize_elevation_request_program()
        .unwrap();

    let outcome = world
        .application
        .compare_and_commit_elevation_request(program, idempotency(71, 71));
    let WorthQueryElevationRequestOutcome::Requested(receipt) = outcome else {
        panic!("the exact request transition must commit: {outcome:?}");
    };

    assert_eq!(receipt.commit_receipt().changed_record_count(), 8);
    assert_eq!(receipt.commit_receipt().emitted_effect_count(), 0);
    assert_eq!(receipt.elevation_key(), "elevation-request-2");
    assert_eq!(receipt.review_key(), "review-request-2");
    assert_eq!(receipt.requester(), principal.principal_entity_id());
    assert_eq!(receipt.elevation_identity(), &string("elevation-2"));
    assert_eq!(receipt.reason(), &string("protect-customer"));
    assert_eq!(receipt.requested_status(), &string("requested"));
    assert_eq!(receipt.issued_at(), &AspectValue::UInt64(100));
    assert_eq!(receipt.expires_at(), &AspectValue::UInt64(105));
    assert_eq!(receipt.review_identity(), &string("review-2"));
    assert_eq!(receipt.review_status(), &string("required"));

    resolve_created_identities(&world, &request);
    world.authorization_time.script([time(100)]);
    let denial = super::admit_raw(&world, &principal, &request, Some("elevation-2"))
        .err()
        .expect("a requested selector cannot bypass the lifecycle transition");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired
    );
}

#[test]
fn ordinary_operation_progression_cannot_authorize_a_lifecycle_request() {
    let world = request_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let access = request_access(&world, &principal, &request, honest_input()).unwrap();
    let operation = request_operation(&world);

    let denial = world
        .application
        .authorize_capability_operation(access, &operation, Default::default())
        .err()
        .expect("the ordinary progression API must reject lifecycle operations");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired
    );
}

#[test]
fn ordinary_compare_and_commit_cannot_publish_a_lifecycle_program() {
    let world = request_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let ordinary = request_reads(&world, &principal, &request, honest_input())
        .begin_effect_program()
        .finish()
        .unwrap();

    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(ordinary, idempotency(72, 72))
    else {
        panic!("ordinary compare-and-commit must reject lifecycle authority");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ElevationTransitionRequired
    );
}

#[test]
fn equivalent_request_retry_recovers_the_same_authoritative_commit() {
    let world = request_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let first = request_reads(&world, &principal, &request, honest_input())
        .materialize_elevation_request_program()
        .unwrap();
    let retry = request_reads(&world, &principal, &request, honest_input())
        .materialize_elevation_request_program()
        .unwrap();

    let first_outcome = world
        .application
        .compare_and_commit_elevation_request(first, idempotency(73, 73));
    let original = match first_outcome {
        WorthQueryElevationRequestOutcome::Requested(original) => original,
        unexpected => panic!("first request must commit: {unexpected:?}"),
    };
    let retry_outcome = world
        .application
        .compare_and_commit_elevation_request(retry, idempotency(73, 73));
    let recovered = match retry_outcome {
        WorthQueryElevationRequestOutcome::AlreadyRequested(recovered) => recovered,
        unexpected => {
            panic!("equivalent request retry must recover the original commit: {unexpected:?}")
        }
    };
    assert!(recovered
        .commit_receipt()
        .is_same_authoritative_commit(original.commit_receipt()));
}

#[test]
fn request_upper_bound_cannot_widen_scope_or_swap_purpose() {
    for (input, expected) in [
        (
            RequestElevationInput {
                target_account: "account-2".to_owned(),
                ..honest_input()
            },
            WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing,
        ),
        (
            RequestElevationInput {
                target_purpose: CapabilityPurpose::Audit,
                ..honest_input()
            },
            WorthQueryOperationAuthorizationDenialKind::ElevationRequestRejected,
        ),
    ] {
        let world = request_world();
        let request = live_scope();
        let principal = authenticated_principal(&world, &request);
        let access = request_access(&world, &principal, &request, input).unwrap();
        let operation = request_operation(&world);

        let denial = world
            .application
            .authorize_elevation_request(access, &operation, Default::default())
            .err()
            .expect("a widened request target must not mint lifecycle authority");
        assert_eq!(denial.kind(), expected);
    }
}

#[test]
fn proposed_grant_must_independently_authorize_the_governed_upper_bound() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::WrongGrant);
    world.authorization_time.script([time(100), time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let access = request_access(
        &world,
        &principal,
        &request,
        RequestElevationInput {
            grant: "capability-2".to_owned(),
            ..honest_input()
        },
    )
    .unwrap();

    let denial = world
        .application
        .authorize_elevation_request(access, &request_operation(&world), Default::default())
        .err()
        .expect("a different grant cannot become the elevation upper bound");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing
    );
    assert!(denial.identity().is_some());
    assert_eq!(
        denial.causes(),
        [WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing]
    );
}

#[test]
fn zero_or_overlong_duration_and_invalid_storage_key_fail_before_commit() {
    for duration in [Duration::ZERO, Duration::from_secs(1_201)] {
        let world = request_world();
        let request = live_scope();
        let principal = authenticated_principal(&world, &request);
        let access = request_access(
            &world,
            &principal,
            &request,
            RequestElevationInput {
                duration,
                ..honest_input()
            },
        )
        .unwrap();
        let denial = world
            .application
            .authorize_elevation_request(access, &request_operation(&world), Default::default())
            .err()
            .expect("invalid duration must not mint request authority");
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::ElevationDurationExceeded
        );
    }

    let world = request_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let access = request_access(
        &world,
        &principal,
        &request,
        RequestElevationInput {
            elevation_key: " bad-key".to_owned(),
            ..honest_input()
        },
    )
    .unwrap();
    let denial = world
        .application
        .authorize_elevation_request(access, &request_operation(&world), Default::default())
        .err()
        .expect("invalid provider key must fail before lifecycle graph progression");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationRequestRejected
    );
}

pub(super) fn request_reads(
    world: &World,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        IdentityExecutionSchema,
        Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: RequestElevationInput,
) -> Reads {
    let operation = request_operation(world);
    let access = request_access(world, principal, request, input).unwrap();
    let admission = world
        .application
        .authorize_elevation_request(access, &operation, Default::default())
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, account| {
            reader
                .require_decision_field(account, AccountLabel::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap()
}

fn request_access(
    world: &World,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        IdentityExecutionSchema,
        Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: RequestElevationInput,
) -> Result<Access, WorthQueryOperationAuthorizationDenial> {
    let capability = world
        .application
        .installed_schema()
        .capability(
            RequestElevationCapability::reference(),
            RequestCapabilityElevationOperation::reference(),
        )
        .unwrap();
    world
        .selected_product()
        .admit_capability_access(principal, &capability, input, request)
}

fn request_operation(
    world: &World,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
    IdentityExecutionSchema,
    RequestCapabilityElevationOperation,
    RequestElevationInput,
> {
    world
        .application
        .installed_schema()
        .installed_operation(RequestCapabilityElevationOperation::reference())
        .unwrap()
}

fn request_world() -> World {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world.authorization_time.hold(time(100));
    world
}

pub(super) fn honest_input() -> RequestElevationInput {
    RequestElevationInput {
        account: "account-1".to_owned(),
        target_account: "account-1".to_owned(),
        grant: "capability-1".to_owned(),
        elevation_key: "elevation-request-2".to_owned(),
        elevation_identity: "elevation-2".to_owned(),
        review_key: "review-request-2".to_owned(),
        review_identity: "review-2".to_owned(),
        reason: "protect-customer".to_owned(),
        duration: Duration::from_secs(5),
        action: CapabilityAction::Touch,
        target_purpose: CapabilityPurpose::AccountMaintenance,
        disclosure: CapabilityDisclosure::AccountActivity,
        amount: 50,
    }
}

pub(super) fn resolve_created_identities(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) {
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            "elevation-2".to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("committed request must create the exact elevation identity");
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityReviewIdentity::reference(),
            "review-2".to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("committed request must create the exact review identity");
}

fn string(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value))
}
