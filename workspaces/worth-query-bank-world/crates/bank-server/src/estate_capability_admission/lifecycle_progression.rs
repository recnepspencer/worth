use std::time::Duration;

use bank_domain::{
    estate::{
        EmergencyAccessId, EmergencyAccessReason, EstateAction, EstateWorkflowStage,
        MandatoryReviewId, RestrictedBankField,
    },
    proposals::BankIdempotencyKey,
};

use super::fixture::{
    capability_world, emergency_request_world, request_scope, GrantSpec,
    APPROVER_UPPER_BOUND_GRANT, ESTATE, GRANT,
};
use super::lifecycle_journey::{
    approve_elevation, request_elevation, ElevationApprovalSpec, ElevationRequestSpec,
};
use crate::estate_progression::BankEstateProgressionDenial;
use crate::{
    BankApprovedEstateElevation, BankEstateElevationApprovalOutcome,
    BankEstateElevationCloseOutcome, BankEstateElevationClosureKind,
    BankEstateElevationRequestOutcome, BankEstateMandatoryReview, BankEstateMandatoryReviewOutcome,
};

#[test]
fn public_bank_runtime_commits_exact_emergency_request_through_query() {
    let fixture = emergency_request_world(
        "estate-emergency-request",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let principal = fixture.authenticate();
    let action = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(301).unwrap(),
        review: MandatoryReviewId::new(302).unwrap(),
        grant: GRANT,
        reason: EmergencyAccessReason::PreventImmediateLoss,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };

    let outcome = fixture
        .runtime
        .request_estate_emergency_access_with_key(
            &principal,
            action,
            &bank_idempotency(31),
            &request_scope(),
        )
        .expect("the public Bank runtime must reach Query's request progression");
    let BankEstateElevationRequestOutcome::Requested(requested) = outcome else {
        panic!("the exact Bank request must commit once: {outcome:?}");
    };

    assert_eq!(requested.elevation_key(), "estate-emergency-access-301");
    assert_eq!(requested.review_key(), "estate-mandatory-review-302");
    assert_eq!(
        requested.request_changed_record_count(),
        8,
        "the request creates its direct authoritative EmergencyEstate relation"
    );
}

#[test]
fn governed_view_grant_cannot_substitute_for_request_command_authority() {
    let fixture = capability_world(
        "estate-emergency-command-substitution",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let principal = fixture.authenticate();
    let action = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(311).unwrap(),
        review: MandatoryReviewId::new(312).unwrap(),
        grant: GRANT,
        reason: EmergencyAccessReason::PreventImmediateLoss,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };

    let denial = fixture
        .runtime
        .request_estate_emergency_access_with_key(
            &principal,
            action,
            &bank_idempotency(41),
            &request_scope(),
        )
        .expect_err("a governed view grant must not authorize the request command");
    let BankEstateProgressionDenial::Authorization(denial) = denial else {
        panic!("command substitution must fail at authorization: {denial:?}");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::CapabilityAuthorizationMissing
    );
}

#[test]
fn request_command_grant_cannot_substitute_for_governed_upper_bound_authority() {
    let fixture = capability_world(
        "estate-emergency-upper-bound-substitution",
        GrantSpec::emergency_request(),
        EstateWorkflowStage::Administration,
    );
    let principal = fixture.authenticate();
    let action = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(321).unwrap(),
        review: MandatoryReviewId::new(322).unwrap(),
        grant: GRANT,
        reason: EmergencyAccessReason::PreventImmediateLoss,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };

    let denial = fixture
        .runtime
        .request_estate_emergency_access_with_key(
            &principal,
            action,
            &bank_idempotency(51),
            &request_scope(),
        )
        .expect_err("a request command grant must not become governed view authority");
    let BankEstateProgressionDenial::Authorization(denial) = denial else {
        panic!("upper-bound substitution must fail at authorization: {denial:?}");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::CapabilityGrantMissing
    );
}

#[test]
fn distinct_approver_commits_the_public_query_approval_transition() {
    let fixture = emergency_request_world(
        "estate-emergency-approval",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let requester = fixture.authenticate();
    let requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 331,
            review: 332,
            idempotency: 61,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    let approver = fixture.authenticate_approver();

    let outcome = fixture
        .runtime
        .approve_estate_emergency_access_with_key(
            &approver,
            requested,
            EstateAction::ApproveEmergencyAccess {
                estate: ESTATE,
                access: EmergencyAccessId::new(331).unwrap(),
            },
            &bank_idempotency(62),
            &request_scope(),
        )
        .expect("a distinct assigned employee should reach Query approval");
    let BankEstateElevationApprovalOutcome::Approved(approved) = outcome else {
        panic!("the exact approval should commit once: {outcome:?}");
    };

    assert!(approved.requester_differs_from_approver());
    assert_eq!(approved.approval_changed_record_count(), 3);
}

#[test]
fn requester_cannot_approve_their_own_elevation() {
    let fixture = emergency_request_world(
        "estate-emergency-self-approval",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let requester = fixture.authenticate();
    let requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 341,
            review: 342,
            idempotency: 71,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    let other_requester = fixture.authenticate_approver();
    let other_requested = request_elevation(
        &fixture,
        &other_requester,
        ElevationRequestSpec {
            grant: APPROVER_UPPER_BOUND_GRANT,
            access: 343,
            review: 344,
            idempotency: 75,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );

    fixture
        .runtime
        .approve_estate_emergency_access_with_key(
            &requester,
            other_requested,
            EstateAction::ApproveEmergencyAccess {
                estate: ESTATE,
                access: EmergencyAccessId::new(343).unwrap(),
            },
            &bank_idempotency(77),
            &request_scope(),
        )
        .expect("the requester independently holds approval-command authority");

    let denial = fixture
        .runtime
        .approve_estate_emergency_access_with_key(
            &requester,
            requested,
            EstateAction::ApproveEmergencyAccess {
                estate: ESTATE,
                access: EmergencyAccessId::new(341).unwrap(),
            },
            &bank_idempotency(72),
            &request_scope(),
        )
        .expect_err("the distinct-actor rule must reject self approval")
        .into_denial();
    let BankEstateProgressionDenial::Authorization(denial) = denial else {
        panic!("self approval must fail during command authorization: {denial:?}");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::DistinctActorRuleMatched
    );
}

#[test]
fn public_bank_runtime_completes_the_exact_close_and_review_lifecycle() {
    let fixture = emergency_request_world(
        "estate-emergency-close-review",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let requester = fixture.authenticate();
    let requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 351,
            review: 352,
            idempotency: 81,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    let approver = fixture.authenticate_approver();
    let approved = approve_elevation(
        &fixture,
        &approver,
        requested,
        ElevationApprovalSpec {
            access: 351,
            idempotency: 83,
        },
    );
    let mandatory = close_approved_elevation(&fixture, &approver, approved);
    complete_required_review(&fixture, mandatory);
}

fn close_approved_elevation(
    fixture: &super::fixture::CapabilityFixture,
    approver: &crate::BankAuthenticatedPrincipal,
    approved: BankApprovedEstateElevation,
) -> BankEstateMandatoryReview {
    let close = fixture
        .runtime
        .revoke_estate_emergency_access_with_key(
            approver,
            approved,
            EstateAction::RevokeEmergencyAccess {
                estate: ESTATE,
                access: EmergencyAccessId::new(351).unwrap(),
            },
            &bank_idempotency(85),
            &request_scope(),
        )
        .expect("the approved elevation should reach Query close");
    let BankEstateElevationCloseOutcome::Closed(mandatory) = close else {
        panic!("the exact close should commit once: {close:?}");
    };
    assert_eq!(
        mandatory.closure_kind(),
        BankEstateElevationClosureKind::Revoked
    );
    assert_eq!(mandatory.close_changed_record_count(), 2);
    mandatory
}

fn complete_required_review(
    fixture: &super::fixture::CapabilityFixture,
    mandatory: BankEstateMandatoryReview,
) {
    let reviewer = fixture.authenticate_reviewer();
    let outcome = fixture
        .runtime
        .complete_estate_mandatory_review_with_key(
            &reviewer,
            mandatory,
            EstateAction::CompleteMandatoryReview {
                estate: ESTATE,
                access: EmergencyAccessId::new(351).unwrap(),
                review: MandatoryReviewId::new(352).unwrap(),
            },
            &bank_idempotency(87),
            &request_scope(),
        )
        .expect("a distinct reviewer should reach Query review completion");
    let BankEstateMandatoryReviewOutcome::Reviewed(reviewed) = outcome else {
        panic!("the exact review should commit once: {outcome:?}");
    };
    assert!(reviewed.reviewer_differs_from_requester());
    assert!(reviewed.reviewer_differs_from_approver());
    assert_eq!(reviewed.review_changed_record_count(), 3);
}

fn bank_idempotency(seed: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("lifecycle-progression-{seed}")).unwrap()
}
