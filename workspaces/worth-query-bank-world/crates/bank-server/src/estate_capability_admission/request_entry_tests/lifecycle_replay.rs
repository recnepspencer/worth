use super::super::fixture::{emergency_request_world_at, AuthorizationTimeController};
use super::*;
use crate::{
    BankAuthorizationDenialKind, BankEstateElevationApprovalOutcome,
    BankEstateElevationCloseOutcome, BankEstateMandatoryReviewOutcome, BankEstateProgressionDenial,
};

#[test]
fn lifecycle_replays_through_query_after_each_prior_transition() {
    let authorization_time = AuthorizationTimeController::at_epoch_seconds(100);
    let fixture = emergency_request_world_at(
        "estate-approval-entry-replay",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
        authorization_time.clone(),
    );
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let scope = request_scope();
    let request_key = BankIdempotencyKey::new("approval-replay-request").unwrap();
    let approval_key = BankIdempotencyKey::new("approval-replay-approval").unwrap();
    let request_action = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(495).unwrap(),
        review: MandatoryReviewId::new(496).unwrap(),
        grant: GRANT,
        reason: EmergencyAccessReason::PreventImmediateLoss,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };
    let request_outcome = fixture
        .runtime
        .request_estate_emergency_access_with_key(&requester, request_action, &request_key, &scope)
        .unwrap();
    let BankEstateElevationRequestOutcome::Requested(requested) = request_outcome else {
        panic!("initial elevation request must commit: {request_outcome:?}");
    };
    let request_data = fixture.lifecycle_original(&requester, request_action, &request_key, &scope);
    assert_eq!(
        request_data.changed_record_count(),
        requested.request_changed_record_count()
    );
    super::lifecycle_observation::assert_selected_original(
        &fixture,
        &requester,
        &scope,
        &request_data,
    );
    let approve_action = EstateAction::ApproveEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(495).unwrap(),
    };
    let BankEstateElevationApprovalOutcome::Approved(first) = fixture
        .runtime
        .approve_estate_emergency_access_with_key(
            &approver,
            requested,
            approve_action,
            &approval_key,
            &scope,
        )
        .unwrap()
    else {
        panic!("first approval must commit");
    };
    let first_commit =
        crate::test_host_policy::with_phase(fixture.runtime.application_runtime(), |phase| {
            fixture
                .runtime
                .request(&approver, &scope)
                .retain_read(phase)
        })
        .unwrap()
        .selected_commit()
        .clone();
    let approval_data =
        fixture.lifecycle_original(&approver, approve_action, &approval_key, &scope);
    assert_eq!(
        approval_data.changed_record_count(),
        first.approval_changed_record_count()
    );
    super::lifecycle_observation::assert_selected_original(
        &fixture,
        &approver,
        &scope,
        &approval_data,
    );
    assert_eq!(
        fixture.lifecycle_original(&approver, approve_action, &approval_key, &scope),
        approval_data
    );
    assert_eq!(
        crate::test_host_policy::with_phase(fixture.runtime.application_runtime(), |phase| {
            fixture
                .runtime
                .request(&approver, &scope)
                .retain_read(phase)
        })
        .unwrap()
        .selected_commit(),
        &first_commit,
    );

    let close_action = EstateAction::RevokeEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(495).unwrap(),
    };
    let close_key = BankIdempotencyKey::new("approval-replay-close").unwrap();
    let close_outcome = fixture
        .runtime
        .revoke_estate_emergency_access_with_key(&approver, first, close_action, &close_key, &scope)
        .unwrap();
    let BankEstateElevationCloseOutcome::Closed(mandatory) = close_outcome else {
        panic!(
            "first close must commit: {}",
            match close_outcome {
                BankEstateElevationCloseOutcome::Denied { kind, stage, .. } =>
                    format!("{kind:?} at {stage:?}"),
                _ => "unexpected outcome".to_string(),
            }
        );
    };
    let first_closed_at = mandatory.closed_at().clone();
    let closed_commit =
        crate::test_host_policy::with_phase(fixture.runtime.application_runtime(), |phase| {
            fixture
                .runtime
                .request(&approver, &scope)
                .retain_read(phase)
        })
        .unwrap()
        .selected_commit()
        .clone();
    let original_close_data =
        fixture.lifecycle_original(&approver, close_action, &close_key, &scope);
    authorization_time.advance_to_epoch_seconds(101);
    let close_data = fixture.lifecycle_original(&approver, close_action, &close_key, &scope);
    assert_eq!(
        close_data.changed_record_count(),
        mandatory.close_changed_record_count()
    );
    assert_eq!(mandatory.closed_at(), &first_closed_at);
    assert_eq!(close_data, original_close_data);
    super::lifecycle_observation::assert_selected_original(
        &fixture,
        &approver,
        &scope,
        &close_data,
    );
    assert_eq!(
        crate::test_host_policy::with_phase(fixture.runtime.application_runtime(), |phase| {
            fixture
                .runtime
                .request(&approver, &scope)
                .retain_read(phase)
        })
        .unwrap()
        .selected_commit(),
        &closed_commit,
    );

    let reviewer = fixture.authenticate_reviewer();
    let review_action = EstateAction::CompleteMandatoryReview {
        estate: ESTATE,
        access: EmergencyAccessId::new(495).unwrap(),
        review: MandatoryReviewId::new(496).unwrap(),
    };
    let review_key = BankIdempotencyKey::new("approval-replay-review").unwrap();
    let BankEstateMandatoryReviewOutcome::Reviewed(reviewed) = fixture
        .runtime
        .complete_estate_mandatory_review_with_key(
            &reviewer,
            mandatory,
            review_action,
            &review_key,
            &scope,
        )
        .unwrap()
    else {
        panic!("first mandatory review must commit");
    };
    let reviewed_commit =
        crate::test_host_policy::with_phase(fixture.runtime.application_runtime(), |phase| {
            fixture
                .runtime
                .request(&reviewer, &scope)
                .retain_read(phase)
        })
        .unwrap()
        .selected_commit()
        .clone();
    let original_review_data =
        fixture.lifecycle_original(&reviewer, review_action, &review_key, &scope);
    let first_reviewed_at = reviewed.reviewed_at().clone();
    authorization_time.advance_to_epoch_seconds(102);
    let review_data = fixture.lifecycle_original(&reviewer, review_action, &review_key, &scope);
    assert_eq!(review_data, original_review_data);
    super::lifecycle_observation::assert_selected_original(
        &fixture,
        &reviewer,
        &scope,
        &review_data,
    );
    assert_eq!(
        crate::test_host_policy::with_phase(fixture.runtime.application_runtime(), |phase| {
            fixture
                .runtime
                .request(&reviewer, &scope)
                .retain_read(phase)
        })
        .unwrap()
        .selected_commit(),
        &reviewed_commit,
    );
    for offset in 0..65_u64 {
        let action = EstateAction::RequestEmergencyAccess {
            estate: ESTATE,
            access: EmergencyAccessId::new(600 + offset).unwrap(),
            review: MandatoryReviewId::new(700 + offset).unwrap(),
            grant: GRANT,
            reason: EmergencyAccessReason::PreventImmediateLoss,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        };
        let key = BankIdempotencyKey::new(format!("replay-eviction-{offset}")).unwrap();
        assert!(matches!(
            fixture
                .runtime
                .request_estate_emergency_access_with_key(&requester, action, &key, &scope),
            Ok(BankEstateElevationRequestOutcome::Requested(_))
        ));
    }
    let unapproved_action = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(800).unwrap(),
        review: MandatoryReviewId::new(801).unwrap(),
        grant: GRANT,
        reason: EmergencyAccessReason::PreventImmediateLoss,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };
    let unapproved_key = BankIdempotencyKey::new("replay-unapproved-expired").unwrap();
    let BankEstateElevationRequestOutcome::Requested(unapproved) = fixture
        .runtime
        .request_estate_emergency_access_with_key(
            &requester,
            unapproved_action,
            &unapproved_key,
            &scope,
        )
        .unwrap()
    else {
        panic!("an unapproved request must commit before the time cutover");
    };
    let after_eviction =
        crate::test_host_policy::with_phase(fixture.runtime.application_runtime(), |phase| {
            fixture
                .runtime
                .request(&reviewer, &scope)
                .retain_read(phase)
        })
        .unwrap()
        .selected_commit()
        .clone();
    authorization_time.advance_to_epoch_seconds(403);
    let unapproved_denial = match fixture.runtime.approve_estate_emergency_access_with_key(
        &approver,
        unapproved,
        EstateAction::ApproveEmergencyAccess {
            estate: ESTATE,
            access: EmergencyAccessId::new(800).unwrap(),
        },
        &BankIdempotencyKey::new("replay-unapproved-approval").unwrap(),
        &scope,
    ) {
        Err(failure) => failure.into_denial(),
        Ok(_) => panic!("an uncommitted approval must fail after its request expires"),
    };
    let BankEstateProgressionDenial::ApprovalAuthorization(denial) = unapproved_denial else {
        panic!("the expired fresh approval must retain its authorization denial");
    };
    assert_eq!(denial.kind(), BankAuthorizationDenialKind::ElevationExpired);
    // Full original data equality includes the Native artifact digest that seals
    // the original timestamp values. No later clock or head becomes receipt truth.
    assert_eq!(
        fixture.lifecycle_original(&approver, close_action, &close_key, &scope),
        close_data
    );
    assert_eq!(
        fixture.lifecycle_original(&reviewer, review_action, &review_key, &scope),
        review_data
    );
    assert_eq!(
        fixture.lifecycle_original(&approver, approve_action, &approval_key, &scope),
        approval_data
    );
    assert_eq!(
        fixture.lifecycle_original(&requester, request_action, &request_key, &scope),
        request_data
    );
    assert_eq!(reviewed.reviewed_at(), &first_reviewed_at);
    assert_eq!(
        crate::test_host_policy::with_phase(fixture.runtime.application_runtime(), |phase| {
            fixture
                .runtime
                .request(&reviewer, &scope)
                .retain_read(phase)
        })
        .unwrap()
        .selected_commit(),
        &after_eviction,
    );
}
