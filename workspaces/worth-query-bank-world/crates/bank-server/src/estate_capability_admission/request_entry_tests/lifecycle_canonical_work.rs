//! Every elevation lifecycle commit, and its exact replay, reports the two
//! request identities its admission derived once each: the input and the key.
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
fn each_lifecycle_commit_and_replay_reports_its_request_identities_once() {
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
    let mut requests = (0..3).map(|_| {
        fixture
            .runtime
            .request_estate_emergency_access_with_key(&requester, request, &key, &scope)
            .unwrap()
    });
    let Some(BankEstateElevationRequestOutcome::Requested(requested)) = requests.next() else {
        panic!("the first request must commit");
    };
    let Some(BankEstateElevationRequestOutcome::AlreadyRequested(replayed_request)) =
        requests.next()
    else {
        panic!("the exact request must replay");
    };
    let Some(BankEstateElevationRequestOutcome::AlreadyRequested(late_request)) = requests.next()
    else {
        panic!("the exact request must replay again");
    };
    // Input 284 encoded, 351 hashed, 6 blocks. Key "lifecycle-work-request"
    // (22 bytes) 109 encoded, 200 hashed with 91 bytes of framing (37-byte
    // `RequestEstateEmergencyAccessOperation`), 4 blocks.
    for requested in [&requested, &replayed_request, &late_request] {
        assert_request_identities(requested.request_canonical_work().admission(), 393, 551, 10);
    }

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
    let BankEstateElevationApprovalOutcome::AlreadyApproved(replayed_approval) =
        approve_with(replayed_request)
    else {
        panic!("the exact approval must replay");
    };
    // Input 92 encoded, 159 hashed, 3 blocks. Key "lifecycle-work-approval"
    // (23 bytes) 111 encoded, 202 hashed with 91 bytes of framing (37-byte
    // `ApproveEstateEmergencyAccessOperation`), 4 blocks.
    for approved in [&approved, &replayed_approval] {
        assert_request_identities(approved.approval_canonical_work().admission(), 203, 361, 7);
    }

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
    let BankEstateElevationCloseOutcome::AlreadyClosed(replayed_mandatory) =
        close_with(replayed_approval)
    else {
        panic!("the exact close must replay");
    };
    // Input 91 encoded, 158 hashed, 3 blocks. Key "lifecycle-work-close"
    // (20 bytes) 108 encoded, 198 hashed with 90 bytes of framing (36-byte
    // `RevokeEstateEmergencyAccessOperation`), 4 blocks.
    for mandatory in [&mandatory, &replayed_mandatory] {
        assert_request_identities(mandatory.close_canonical_work().admission(), 199, 356, 7);
    }
    // Past the request's expiry a fresh approval is denied, so this replay
    // recovers the approval after that denial and still reports both
    // derivations.
    time.advance_to_epoch_seconds(401);
    let BankEstateElevationApprovalOutcome::AlreadyApproved(late_approval) =
        approve_with(late_request)
    else {
        panic!("the exact approval must replay after the close");
    };
    assert_request_identities(
        late_approval.approval_canonical_work().admission(),
        203,
        361,
        7,
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
    let BankEstateMandatoryReviewOutcome::AlreadyReviewed(replayed_review) =
        review_with(replayed_mandatory)
    else {
        panic!("the exact review must replay");
    };
    // Input 123 encoded, 190 hashed, 4 blocks. Key "lifecycle-work-review"
    // (21 bytes) 109 encoded, 201 hashed with 92 bytes of framing (38-byte
    // `CompleteEstateMandatoryReviewOperation`), 4 blocks.
    for reviewed in [&reviewed, &replayed_review] {
        assert_request_identities(reviewed.review_canonical_work().admission(), 232, 391, 8);
    }
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
