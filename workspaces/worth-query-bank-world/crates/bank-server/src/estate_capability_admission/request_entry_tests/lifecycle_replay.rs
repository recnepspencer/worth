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
    let BankEstateElevationRequestOutcome::AlreadyRequested(replayed_request) = fixture
        .runtime
        .request_estate_emergency_access_with_key(&requester, request_action, &request_key, &scope)
        .unwrap()
    else {
        panic!("the exact request must return a second authority");
    };
    let BankEstateElevationRequestOutcome::AlreadyRequested(late_close_request) = fixture
        .runtime
        .request_estate_emergency_access_with_key(&requester, request_action, &request_key, &scope)
        .unwrap()
    else {
        panic!("the exact request must retain authority for later close replay");
    };
    let BankEstateElevationRequestOutcome::AlreadyRequested(late_approval_request) = fixture
        .runtime
        .request_estate_emergency_access_with_key(&requester, request_action, &request_key, &scope)
        .unwrap()
    else {
        panic!("the exact request must retain authority for later approval replay");
    };
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
    let first_commit = fixture
        .runtime
        .request(&approver, &scope)
        .retain_read()
        .unwrap()
        .selected_commit()
        .clone();
    let replay = fixture
        .runtime
        .approve_estate_emergency_access_with_key(
            &approver,
            replayed_request,
            approve_action,
            &approval_key,
            &scope,
        )
        .unwrap();
    let BankEstateElevationApprovalOutcome::AlreadyApproved(replayed) = replay else {
        panic!("Query must return the committed approval on exact replay: {replay:?}");
    };
    let BankEstateElevationApprovalOutcome::AlreadyApproved(late_close_approved) = fixture
        .runtime
        .approve_estate_emergency_access_with_key(
            &approver,
            late_close_request,
            approve_action,
            &approval_key,
            &scope,
        )
        .unwrap()
    else {
        panic!("exact approval replay must retain authority for later close replay");
    };
    assert_eq!(replayed.query().approver(), first.query().approver());
    assert_eq!(
        fixture
            .runtime
            .request(&approver, &scope)
            .retain_read()
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
    let closed_commit = fixture
        .runtime
        .request(&approver, &scope)
        .retain_read()
        .unwrap()
        .selected_commit()
        .clone();
    authorization_time.advance_to_epoch_seconds(101);
    let close_replay = fixture
        .runtime
        .revoke_estate_emergency_access_with_key(
            &approver,
            replayed,
            close_action,
            &close_key,
            &scope,
        )
        .unwrap();
    let BankEstateElevationCloseOutcome::AlreadyClosed(replayed_mandatory) = close_replay else {
        panic!("Query must return the committed close on exact replay: {close_replay:?}");
    };
    assert_eq!(replayed_mandatory.closure_kind(), mandatory.closure_kind());
    assert_eq!(replayed_mandatory.closed_at(), mandatory.closed_at());
    assert_eq!(
        fixture
            .runtime
            .request(&approver, &scope)
            .retain_read()
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
    let reviewed_commit = fixture
        .runtime
        .request(&reviewer, &scope)
        .retain_read()
        .unwrap()
        .selected_commit()
        .clone();
    authorization_time.advance_to_epoch_seconds(102);
    let review_replay = fixture
        .runtime
        .complete_estate_mandatory_review_with_key(
            &reviewer,
            replayed_mandatory,
            review_action,
            &review_key,
            &scope,
        )
        .unwrap();
    let BankEstateMandatoryReviewOutcome::AlreadyReviewed(replayed_review) = review_replay else {
        panic!("Query must return the committed review on exact replay: {review_replay:?}");
    };
    assert_eq!(replayed_review.closure_kind(), reviewed.closure_kind());
    assert_eq!(replayed_review.reviewed_at(), reviewed.reviewed_at());
    assert_eq!(
        fixture
            .runtime
            .request(&reviewer, &scope)
            .retain_read()
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
    let after_eviction = fixture
        .runtime
        .request(&reviewer, &scope)
        .retain_read()
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
    let later_close = fixture.runtime.revoke_estate_emergency_access_with_key(
        &approver,
        late_close_approved,
        close_action,
        &close_key,
        &scope,
    );
    let BankEstateElevationCloseOutcome::AlreadyClosed(later_mandatory) = later_close.unwrap()
    else {
        panic!("exact close replay must survive the later review transition");
    };
    assert_eq!(later_mandatory.closed_at(), &first_closed_at);
    assert_eq!(later_mandatory.closure_kind(), reviewed.closure_kind());
    let later_review = fixture.runtime.complete_estate_mandatory_review_with_key(
        &reviewer,
        later_mandatory,
        review_action,
        &review_key,
        &scope,
    );
    let BankEstateMandatoryReviewOutcome::AlreadyReviewed(later_reviewed) = later_review.unwrap()
    else {
        panic!("exact review replay must survive commit-basis eviction");
    };
    assert_eq!(later_reviewed.reviewed_at(), reviewed.reviewed_at());
    let later_approval = fixture.runtime.approve_estate_emergency_access_with_key(
        &approver,
        late_approval_request,
        approve_action,
        &approval_key,
        &scope,
    );
    assert!(
        matches!(
            later_approval,
            Ok(BankEstateElevationApprovalOutcome::AlreadyApproved(_))
        ),
        "exact approval replay must survive later close and review: {later_approval:?}"
    );
    assert_eq!(
        fixture
            .runtime
            .request(&reviewer, &scope)
            .retain_read()
            .unwrap()
            .selected_commit(),
        &after_eviction,
    );
}
