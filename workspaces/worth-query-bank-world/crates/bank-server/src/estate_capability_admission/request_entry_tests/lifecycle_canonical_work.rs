//! Fresh lifecycle commits report the two admission identities once each.
//! Recorded retries preserve original meaning and expose no live work counters.
//!
//! Each derivation hashes its encoded bytes behind framing of 16 bytes of
//! lengths plus its domain and scope. The input is framed with the 30-byte
//! input domain and the 21-byte `bank.estate.action.v1` (67 bytes). The key is
//! framed with the 38-byte capability workflow key domain and the operation
//! identifier, and encodes the principal basis (87 bytes of key overhead for
//! principal 7, 88 for principals 13 and 15, whose ids have two digits) plus
//! the client key text. Blocks are ceil((hashed + 9) / 64) per derivation.

use super::super::fixture::{emergency_request_world_at, AuthorizationTimeController};
use super::*;
use crate::{
    BankCommitCanonicalWorkEvidence, BankEstateElevationApprovalOutcome,
    BankEstateElevationCloseOutcome, BankEstateMandatoryReviewOutcome,
};

#[test]
fn lifecycle_commits_report_request_work_and_retries_preserve_original_meaning() {
    let time = AuthorizationTimeController::at_epoch_seconds(100);
    let fixture = emergency_request_world_at(
        "estate-lifecycle-canonical-work",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
        time.clone(),
    );
    let (requester, approver) = (fixture.authenticate(), fixture.authenticate_approver());
    let reviewer = fixture.authenticate_reviewer();
    let scope = request_scope();
    let (access, review) = (
        EmergencyAccessId::new(497).unwrap(),
        MandatoryReviewId::new(498).unwrap(),
    );
    let request = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access,
        review,
        grant: GRANT,
        reason: EmergencyAccessReason::PreventImmediateLoss,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };
    let key = BankIdempotencyKey::new("lifecycle-work-request").unwrap();
    let BankEstateElevationRequestOutcome::Requested(requested) = fixture
        .runtime
        .request_estate_emergency_access_with_key(&requester, request, &key, &scope)
        .unwrap()
    else {
        panic!("the first request must commit");
    };
    let request_data = fixture.lifecycle_original(&requester, request, &key, &scope);
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
    assert_eq!(
        fixture.lifecycle_original(&requester, request, &key, &scope),
        request_data
    );
    // Input 284 encoded, 351 hashed, 6 blocks. Key "lifecycle-work-request"
    // (22 bytes) 109 encoded, 200 hashed with 91 bytes of framing (37-byte
    // `RequestEstateEmergencyAccessOperation`), 4 blocks.
    assert_request_identities(requested.request_canonical_work().admission(), 393, 551, 10);

    let approve = EstateAction::ApproveEmergencyAccess {
        estate: ESTATE,
        access,
    };
    let key = BankIdempotencyKey::new("lifecycle-work-approval").unwrap();
    let approve_with = |authority| {
        fixture
            .runtime
            .approve_estate_emergency_access_with_key(&approver, authority, approve, &key, &scope)
            .unwrap()
    };
    let BankEstateElevationApprovalOutcome::Approved(approved) = approve_with(requested) else {
        panic!("the first approval must commit");
    };
    let approval_key = key.clone();
    let approval_data = fixture.lifecycle_original(&approver, approve, &key, &scope);
    assert_eq!(
        approval_data.changed_record_count(),
        approved.approval_changed_record_count()
    );
    super::lifecycle_observation::assert_selected_original(
        &fixture,
        &approver,
        &scope,
        &approval_data,
    );
    assert_eq!(
        fixture.lifecycle_original(&approver, approve, &key, &scope),
        approval_data
    );
    // Input 92 encoded, 159 hashed, 3 blocks. Key "lifecycle-work-approval"
    // (23 bytes) 111 encoded, 202 hashed with 91 bytes of framing (37-byte
    // `ApproveEstateEmergencyAccessOperation`), 4 blocks.
    assert_request_identities(approved.approval_canonical_work().admission(), 203, 361, 7);

    let close = EstateAction::RevokeEmergencyAccess {
        estate: ESTATE,
        access,
    };
    let key = BankIdempotencyKey::new("lifecycle-work-close").unwrap();
    let close_with = |authority| {
        fixture
            .runtime
            .revoke_estate_emergency_access_with_key(&approver, authority, close, &key, &scope)
            .unwrap()
    };
    let BankEstateElevationCloseOutcome::Closed(mandatory) = close_with(approved) else {
        panic!("the first close must commit");
    };
    let close_data = fixture.lifecycle_original(&approver, close, &key, &scope);
    assert_eq!(
        close_data.changed_record_count(),
        mandatory.close_changed_record_count()
    );
    super::lifecycle_observation::assert_selected_original(
        &fixture,
        &approver,
        &scope,
        &close_data,
    );
    assert_eq!(
        fixture.lifecycle_original(&approver, close, &key, &scope),
        close_data
    );
    // Input 91 encoded, 158 hashed, 3 blocks. Key "lifecycle-work-close"
    // (20 bytes) 108 encoded, 198 hashed with 90 bytes of framing (36-byte
    // `RevokeEstateEmergencyAccessOperation`), 4 blocks.
    assert_request_identities(mandatory.close_canonical_work().admission(), 199, 356, 7);
    // Recorded data cannot produce a second phase owner. Its exact original
    // artifact remains observable after later transitions and request expiry.
    time.advance_to_epoch_seconds(401);
    assert_eq!(
        fixture.lifecycle_original(&approver, approve, &approval_key, &scope),
        approval_data
    );
    assert_eq!(
        fixture.lifecycle_head(&approver, &scope),
        close_data
            .new_product_head()
            .memory()
            .unwrap()
            .selected_commit()
    );

    let complete = EstateAction::CompleteMandatoryReview {
        estate: ESTATE,
        access,
        review,
    };
    let key = BankIdempotencyKey::new("lifecycle-work-review").unwrap();
    let review_with = |authority| {
        fixture
            .runtime
            .complete_estate_mandatory_review_with_key(&reviewer, authority, complete, &key, &scope)
            .unwrap()
    };
    let BankEstateMandatoryReviewOutcome::Reviewed(reviewed) = review_with(mandatory) else {
        panic!("the first review must commit");
    };
    let review_data = fixture.lifecycle_original(&reviewer, complete, &key, &scope);
    assert_eq!(
        review_data.changed_record_count(),
        reviewed.review_changed_record_count()
    );
    super::lifecycle_observation::assert_selected_original(
        &fixture,
        &reviewer,
        &scope,
        &review_data,
    );
    assert_eq!(
        fixture.lifecycle_original(&reviewer, complete, &key, &scope),
        review_data
    );
    // Input 123 encoded, 190 hashed, 4 blocks. Key "lifecycle-work-review"
    // (21 bytes) 109 encoded, 201 hashed with 92 bytes of framing (38-byte
    // `CompleteEstateMandatoryReviewOperation`), 4 blocks.
    assert_request_identities(reviewed.review_canonical_work().admission(), 232, 391, 8);
}

fn assert_request_identities(
    admission: BankCommitCanonicalWorkEvidence,
    encoded: usize,
    hashed: usize,
    blocks: usize,
) {
    assert_eq!(admission.basis_preparations(), 0);
    assert_eq!(
        admission.digest_derivations(),
        2,
        "one key and one input derivation"
    );
    assert_eq!(admission.canonical_entries(), 2);
    assert_eq!(admission.canonical_encoded_bytes(), encoded);
    assert_eq!(admission.canonical_material_allocation_bytes(), 0);
    assert_eq!(admission.sha256_input_bytes(), hashed);
    assert_eq!(admission.sha256_compression_blocks(), blocks);
    assert_eq!(admission.digest_text_materializations(), 0);
}
