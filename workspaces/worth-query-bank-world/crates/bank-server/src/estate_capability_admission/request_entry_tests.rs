use std::time::Duration;

use bank_domain::{
    estate::{
        EmergencyAccessId, EmergencyAccessReason, EstateAction, EstateWorkflowStage,
        MandatoryReviewId, RestrictedBankField,
    },
    proposals::BankIdempotencyKey,
};

use super::fixture::{
    emergency_request_world, request_scope, GrantSpec, APPROVER_UPPER_BOUND_GRANT, ESTATE, GRANT,
};
use crate::{BankCommitDenialKind, BankEstateElevationRequestOutcome};

#[test]
fn request_entry_replays_exact_input_and_denies_changed_reason() {
    let fixture = emergency_request_world(
        "estate-request-entry-idempotency",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let principal = fixture.authenticate();
    let scope = request_scope();
    let key = BankIdempotencyKey::new("request-entry-idempotency").unwrap();
    let action = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(491).unwrap(),
        review: MandatoryReviewId::new(492).unwrap(),
        grant: GRANT,
        reason: EmergencyAccessReason::PreventImmediateLoss,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };
    let first = fixture
        .runtime
        .request_estate_emergency_access_with_key(&principal, action, &key, &scope)
        .unwrap();
    let BankEstateElevationRequestOutcome::Requested(requested) = first else {
        panic!("the first request must publish: {first:?}");
    };
    assert_eq!(requested.request_changed_record_count(), 8);
    let first_commit = fixture
        .runtime
        .request(&principal, &scope)
        .retain_read()
        .unwrap()
        .selected_commit()
        .clone();

    let retry = fixture
        .runtime
        .request_estate_emergency_access_with_key(&principal, action, &key, &scope)
        .unwrap();
    let BankEstateElevationRequestOutcome::AlreadyRequested(replayed) = retry else {
        panic!("the exact request must replay: {retry:?}");
    };
    assert_eq!(replayed.elevation_key(), requested.elevation_key());
    assert_eq!(replayed.query().requester(), requested.query().requester());

    let changed = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(491).unwrap(),
        review: MandatoryReviewId::new(492).unwrap(),
        grant: GRANT,
        reason: EmergencyAccessReason::MeetLegalDeadline,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };
    let drift = fixture
        .runtime
        .request_estate_emergency_access_with_key(&principal, changed, &key, &scope)
        .unwrap();
    assert!(matches!(
        drift,
        BankEstateElevationRequestOutcome::Denied {
            kind: BankCommitDenialKind::IdempotencyIntentDrift,
            ..
        }
    ));
    assert_eq!(
        fixture
            .runtime
            .request(&principal, &scope)
            .retain_read()
            .unwrap()
            .selected_commit(),
        &first_commit,
        "replay and drift must leave the first publication as the only effect"
    );

    let approver = fixture.authenticate_approver();
    let second_actor_action = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(493).unwrap(),
        review: MandatoryReviewId::new(494).unwrap(),
        grant: APPROVER_UPPER_BOUND_GRANT,
        reason: EmergencyAccessReason::PreventImmediateLoss,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };
    let second_actor = fixture
        .runtime
        .request_estate_emergency_access_with_key(&approver, second_actor_action, &key, &scope)
        .unwrap();
    let BankEstateElevationRequestOutcome::Requested(second_requested) = second_actor else {
        panic!("a separate principal must own the same client key independently: {second_actor:?}");
    };
    assert_ne!(
        second_requested.query().requester(),
        requested.query().requester()
    );
}

mod lifecycle_replay;
