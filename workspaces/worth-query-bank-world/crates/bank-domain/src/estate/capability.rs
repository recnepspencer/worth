use crate::model::{AccountId, BankPrincipalId, InstitutionId, Money, USD};

use super::{
    BranchId, CapabilityGrantId, DelegationLimit, EstateCaseId, EstateMoment, EstateWorkflowStage,
    RestrictedBankField,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum EstateCapabilityOperation {
    NotifyDeath,
    RetransmitDeathNotice,
    FreezeAccount,
    OpenEstateCase,
    RecognizeExecutor,
    DelegateCapability,
    RevokeCapability,
    RequestEmergencyAccess,
    ApproveEmergencyAccess,
    RevokeEmergencyAccess,
    CompleteMandatoryReview,
    ReleaseEstate,
    DisburseEstate,
    ViewRestrictedEstate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum EstateCapabilityPurpose {
    EstateAdministration,
    IdentityVerification,
    LegalCompliance,
    EmergencyProtection,
    EstateDisbursement,
    MandatoryReview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityGrantStatus {
    Active,
    Revoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CapabilityValidity {
    not_before: EstateMoment,
    not_after: EstateMoment,
}

impl CapabilityValidity {
    pub const fn new(not_before: EstateMoment, not_after: EstateMoment) -> Option<Self> {
        if not_before.epoch_seconds() > not_after.epoch_seconds() {
            None
        } else {
            Some(Self {
                not_before,
                not_after,
            })
        }
    }

    pub const fn contains(self, moment: EstateMoment) -> bool {
        self.not_before.epoch_seconds() <= moment.epoch_seconds()
            && moment.epoch_seconds() <= self.not_after.epoch_seconds()
    }

    pub const fn not_before(self) -> EstateMoment {
        self.not_before
    }

    pub const fn not_after(self) -> EstateMoment {
        self.not_after
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct EstateCapabilityScope {
    pub account: Option<AccountId>,
    pub estate: EstateCaseId,
    pub institution: InstitutionId,
    pub branch: BranchId,
    pub operation: EstateCapabilityOperation,
    pub purpose: EstateCapabilityPurpose,
    pub field: Option<RestrictedBankField>,
    pub amount_ceiling: Option<Money<USD>>,
    pub validity: CapabilityValidity,
    pub delegation: DelegationLimit,
    pub workflow_stage: EstateWorkflowStage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct EstateCapabilityDelegationRequest {
    pub id: CapabilityGrantId,
    pub grantee: BankPrincipalId,
    pub scope: EstateCapabilityScope,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateCapabilityGrant {
    pub id: CapabilityGrantId,
    pub grantor: BankPrincipalId,
    pub grantee: BankPrincipalId,
    pub scope: EstateCapabilityScope,
    pub parent: Option<CapabilityGrantId>,
    pub status: CapabilityGrantStatus,
}
