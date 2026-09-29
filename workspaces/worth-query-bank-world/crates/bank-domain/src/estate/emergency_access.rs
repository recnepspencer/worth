use crate::model::BankPrincipalId;

use super::{CapabilityGrantId, EmergencyAccessId, EstateMoment, MandatoryReviewId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum EmergencyAccessReason {
    PreventImmediateLoss,
    ProtectVulnerableCustomer,
    MeetLegalDeadline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmergencyAccessStatus {
    Requested,
    Approved,
    Expired,
    Revoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateEmergencyAccess {
    pub id: EmergencyAccessId,
    pub requester: BankPrincipalId,
    pub approver: Option<BankPrincipalId>,
    pub reviewer: Option<BankPrincipalId>,
    pub grant: CapabilityGrantId,
    pub review: MandatoryReviewId,
    pub reason: EmergencyAccessReason,
    pub status: EmergencyAccessStatus,
    pub issued_at: EstateMoment,
    pub expires_at: EstateMoment,
}
