use crate::model::{AccountId, BankPrincipalId, Money, SignedMoney, USD};

use super::{
    CapabilityGrantId, DeathNoticeId, EmergencyAccessId, EmergencyAccessReason,
    EstateCapabilityDelegationRequest, EstateCapabilityOperation, EstateCapabilityPurpose,
    EstateCaseId, LegalAuthorityId, MandatoryReviewId, RestrictedBankField,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct EstatePosting {
    pub account: AccountId,
    pub amount: SignedMoney<USD>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct EstateDisbursement {
    pub estate: EstateCaseId,
    pub source_account: AccountId,
    pub destination_account: AccountId,
    pub beneficiary: BankPrincipalId,
    pub amount: Money<USD>,
    pub postings: [EstatePosting; 2],
}

impl EstateDisbursement {
    pub fn new(
        estate: EstateCaseId,
        source_account: AccountId,
        destination_account: AccountId,
        beneficiary: BankPrincipalId,
        amount: Money<USD>,
    ) -> Result<Self, EstateDisbursementInputError> {
        if source_account == destination_account {
            return Err(EstateDisbursementInputError::SameAccount);
        }
        let amount_minor = amount.minor_units();
        Ok(Self {
            estate,
            source_account,
            destination_account,
            beneficiary,
            amount,
            postings: [
                EstatePosting {
                    account: source_account,
                    amount: SignedMoney::from_minor(-amount_minor),
                },
                EstatePosting {
                    account: destination_account,
                    amount: SignedMoney::from_minor(amount_minor),
                },
            ],
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EstateDisbursementInputError {
    SameAccount,
}

impl std::fmt::Display for EstateDisbursementInputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("estate disbursement accounts must be distinct")
    }
}

impl std::error::Error for EstateDisbursementInputError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum EstateAction {
    NotifyDeath {
        estate: EstateCaseId,
        notice: DeathNoticeId,
        subject: BankPrincipalId,
    },
    RetransmitDeathNotice {
        estate: EstateCaseId,
        notice: DeathNoticeId,
        subject: BankPrincipalId,
    },
    FreezeAccount {
        estate: EstateCaseId,
        account: AccountId,
    },
    OpenEstateCase {
        estate: EstateCaseId,
        notice: DeathNoticeId,
    },
    RecognizeExecutor {
        estate: EstateCaseId,
        executor: BankPrincipalId,
        authority: LegalAuthorityId,
    },
    DelegateCapability {
        estate: EstateCaseId,
        parent: CapabilityGrantId,
        child: EstateCapabilityDelegationRequest,
    },
    RevokeCapability {
        estate: EstateCaseId,
        grant: CapabilityGrantId,
    },
    RequestEmergencyAccess {
        estate: EstateCaseId,
        access: EmergencyAccessId,
        review: MandatoryReviewId,
        grant: CapabilityGrantId,
        reason: EmergencyAccessReason,
        field: RestrictedBankField,
        duration: std::time::Duration,
    },
    ApproveEmergencyAccess {
        estate: EstateCaseId,
        access: EmergencyAccessId,
    },
    RevokeEmergencyAccess {
        estate: EstateCaseId,
        access: EmergencyAccessId,
    },
    CompleteMandatoryReview {
        estate: EstateCaseId,
        access: EmergencyAccessId,
        review: MandatoryReviewId,
    },
    ReleaseEstate {
        estate: EstateCaseId,
        executor: BankPrincipalId,
        authority: LegalAuthorityId,
        review: MandatoryReviewId,
    },
    DisburseEstate(EstateDisbursement),
    ViewRestrictedEstate {
        estate: EstateCaseId,
        field: RestrictedBankField,
        purpose: EstateCapabilityPurpose,
    },
    ViewRestrictedEstateWithEmergencyAccess {
        estate: EstateCaseId,
        access: EmergencyAccessId,
        field: RestrictedBankField,
    },
}

impl EstateAction {
    pub const fn operation(self) -> EstateCapabilityOperation {
        match self {
            Self::NotifyDeath { .. } => EstateCapabilityOperation::NotifyDeath,
            Self::RetransmitDeathNotice { .. } => EstateCapabilityOperation::RetransmitDeathNotice,
            Self::FreezeAccount { .. } => EstateCapabilityOperation::FreezeAccount,
            Self::OpenEstateCase { .. } => EstateCapabilityOperation::OpenEstateCase,
            Self::RecognizeExecutor { .. } => EstateCapabilityOperation::RecognizeExecutor,
            Self::DelegateCapability { .. } => EstateCapabilityOperation::DelegateCapability,
            Self::RevokeCapability { .. } => EstateCapabilityOperation::RevokeCapability,
            Self::RequestEmergencyAccess { .. } => {
                EstateCapabilityOperation::RequestEmergencyAccess
            }
            Self::ApproveEmergencyAccess { .. } => {
                EstateCapabilityOperation::ApproveEmergencyAccess
            }
            Self::RevokeEmergencyAccess { .. } => EstateCapabilityOperation::RevokeEmergencyAccess,
            Self::CompleteMandatoryReview { .. } => {
                EstateCapabilityOperation::CompleteMandatoryReview
            }
            Self::ReleaseEstate { .. } => EstateCapabilityOperation::ReleaseEstate,
            Self::DisburseEstate(_) => EstateCapabilityOperation::DisburseEstate,
            Self::ViewRestrictedEstate { .. }
            | Self::ViewRestrictedEstateWithEmergencyAccess { .. } => {
                EstateCapabilityOperation::ViewRestrictedEstate
            }
        }
    }

    pub const fn estate(self) -> Option<EstateCaseId> {
        match self {
            Self::NotifyDeath { estate, .. }
            | Self::RetransmitDeathNotice { estate, .. }
            | Self::FreezeAccount { estate, .. }
            | Self::OpenEstateCase { estate, .. }
            | Self::RecognizeExecutor { estate, .. }
            | Self::DelegateCapability { estate, .. }
            | Self::RevokeCapability { estate, .. }
            | Self::RequestEmergencyAccess { estate, .. }
            | Self::ApproveEmergencyAccess { estate, .. }
            | Self::RevokeEmergencyAccess { estate, .. }
            | Self::CompleteMandatoryReview { estate, .. }
            | Self::ReleaseEstate { estate, .. }
            | Self::ViewRestrictedEstate { estate, .. }
            | Self::ViewRestrictedEstateWithEmergencyAccess { estate, .. } => Some(estate),
            Self::DisburseEstate(disbursement) => Some(disbursement.estate),
        }
    }

    pub const fn account(self) -> Option<AccountId> {
        match self {
            Self::FreezeAccount { account, .. } => Some(account),
            Self::DisburseEstate(disbursement) => Some(disbursement.source_account),
            _ => None,
        }
    }

    pub const fn field(self) -> Option<RestrictedBankField> {
        match self {
            Self::ViewRestrictedEstate { field, .. }
            | Self::ViewRestrictedEstateWithEmergencyAccess { field, .. } => Some(field),
            _ => None,
        }
    }

    pub const fn purpose(self) -> EstateCapabilityPurpose {
        match self {
            Self::DisburseEstate(_) => EstateCapabilityPurpose::EstateDisbursement,
            Self::ViewRestrictedEstate { purpose, .. } => purpose,
            Self::ViewRestrictedEstateWithEmergencyAccess { .. } => {
                EstateCapabilityPurpose::EmergencyProtection
            }
            Self::CompleteMandatoryReview { .. } => EstateCapabilityPurpose::MandatoryReview,
            Self::RequestEmergencyAccess { .. }
            | Self::ApproveEmergencyAccess { .. }
            | Self::RevokeEmergencyAccess { .. } => EstateCapabilityPurpose::EmergencyProtection,
            Self::RecognizeExecutor { .. } => EstateCapabilityPurpose::LegalCompliance,
            _ => EstateCapabilityPurpose::EstateAdministration,
        }
    }

    pub const fn amount(self) -> Option<Money<USD>> {
        match self {
            Self::DisburseEstate(disbursement) => Some(disbursement.amount),
            _ => None,
        }
    }
}
