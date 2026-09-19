use std::time::Duration;

use bank_domain::{
    estate::{
        CapabilityGrantId, EmergencyAccessId, EmergencyAccessReason, EstateAction,
        MandatoryReviewId, RestrictedBankField,
    },
    proposals::BankIdempotencyKey,
};

use super::fixture::{request_scope, CapabilityFixture, ESTATE};
use crate::{
    BankApprovedEstateElevation, BankAuthenticatedPrincipal, BankEstateElevationApprovalOutcome,
    BankEstateElevationRequestOutcome, BankRequestedEstateElevation,
};

pub(super) struct ElevationRequestSpec {
    pub(super) grant: CapabilityGrantId,
    pub(super) access: u64,
    pub(super) review: u64,
    pub(super) idempotency: u8,
    pub(super) field: RestrictedBankField,
    pub(super) duration: Duration,
}

pub(super) struct ElevationApprovalSpec {
    pub(super) access: u64,
    pub(super) idempotency: u8,
}

pub(super) fn request_elevation(
    fixture: &CapabilityFixture,
    requester: &BankAuthenticatedPrincipal,
    spec: ElevationRequestSpec,
) -> BankRequestedEstateElevation {
    let outcome = fixture
        .runtime
        .request_estate_emergency_access_with_key(
            requester,
            EstateAction::RequestEmergencyAccess {
                estate: ESTATE,
                access: EmergencyAccessId::new(spec.access).unwrap(),
                review: MandatoryReviewId::new(spec.review).unwrap(),
                grant: spec.grant,
                reason: EmergencyAccessReason::PreventImmediateLoss,
                field: spec.field,
                duration: spec.duration,
            },
            &bank_idempotency(spec.idempotency),
            &request_scope(),
        )
        .expect("the approval prerequisite request should commit");
    let BankEstateElevationRequestOutcome::Requested(requested) = outcome else {
        panic!("the approval prerequisite must be fresh: {outcome:?}");
    };
    requested
}

pub(super) fn approve_elevation(
    fixture: &CapabilityFixture,
    approver: &BankAuthenticatedPrincipal,
    requested: BankRequestedEstateElevation,
    spec: ElevationApprovalSpec,
) -> BankApprovedEstateElevation {
    let outcome = fixture
        .runtime
        .approve_estate_emergency_access_with_key(
            approver,
            requested,
            EstateAction::ApproveEmergencyAccess {
                estate: ESTATE,
                access: EmergencyAccessId::new(spec.access).unwrap(),
            },
            &bank_idempotency(spec.idempotency),
            &request_scope(),
        )
        .expect("the terminal lifecycle prerequisite approval should commit");
    let BankEstateElevationApprovalOutcome::Approved(approved) = outcome else {
        panic!("the terminal prerequisite approval must be fresh: {outcome:?}");
    };
    approved
}

fn bank_idempotency(seed: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("estate-elevation-{seed}")).unwrap()
}
