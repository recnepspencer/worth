use crate::model::{AccountId, BankPrincipalId};

use super::{
    EmergencyAccessId, EstateCapabilityPurpose, EstateCaseId, EstatePosting, LegalAuthorityId,
    MandatoryReviewId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankDisclosureClassification {
    Restricted,
    HighlyRestricted,
    LegalSealed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum RestrictedBankField {
    CustomerIdentity,
    BeneficiaryIdentity,
    LegalDocument,
    AccountDetails,
    PostingHistory,
    AuditTrail,
    GovernanceMetadata,
    EmergencyAccessActivity,
}

impl RestrictedBankField {
    pub const ALL: [Self; 8] = [
        Self::CustomerIdentity,
        Self::BeneficiaryIdentity,
        Self::LegalDocument,
        Self::AccountDetails,
        Self::PostingHistory,
        Self::AuditTrail,
        Self::GovernanceMetadata,
        Self::EmergencyAccessActivity,
    ];

    pub const fn classification(self) -> BankDisclosureClassification {
        match self {
            Self::CustomerIdentity | Self::AccountDetails => {
                BankDisclosureClassification::Restricted
            }
            Self::BeneficiaryIdentity
            | Self::PostingHistory
            | Self::AuditTrail
            | Self::GovernanceMetadata
            | Self::EmergencyAccessActivity => BankDisclosureClassification::HighlyRestricted,
            Self::LegalDocument => BankDisclosureClassification::LegalSealed,
        }
    }

    pub const fn permits(self, purpose: EstateCapabilityPurpose) -> bool {
        match self {
            Self::CustomerIdentity => matches!(
                purpose,
                EstateCapabilityPurpose::EstateAdministration
                    | EstateCapabilityPurpose::IdentityVerification
                    | EstateCapabilityPurpose::LegalCompliance
            ),
            Self::BeneficiaryIdentity | Self::AccountDetails => matches!(
                purpose,
                EstateCapabilityPurpose::EstateAdministration
                    | EstateCapabilityPurpose::LegalCompliance
                    | EstateCapabilityPurpose::EmergencyProtection
            ),
            Self::LegalDocument => {
                matches!(purpose, EstateCapabilityPurpose::LegalCompliance)
            }
            Self::PostingHistory => matches!(
                purpose,
                EstateCapabilityPurpose::EstateAdministration
                    | EstateCapabilityPurpose::LegalCompliance
                    | EstateCapabilityPurpose::MandatoryReview
            ),
            Self::AuditTrail => matches!(
                purpose,
                EstateCapabilityPurpose::LegalCompliance | EstateCapabilityPurpose::MandatoryReview
            ),
            Self::GovernanceMetadata => {
                matches!(purpose, EstateCapabilityPurpose::EstateAdministration)
            }
            Self::EmergencyAccessActivity => {
                matches!(purpose, EstateCapabilityPurpose::EmergencyProtection)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankDisclosure<T> {
    Disclosed(T),
    Omitted(BankDisclosureClassification),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstatePostingHistory {
    pub postings: [EstatePosting; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateAuditTrail {
    pub emergency_access: Option<EmergencyAccessId>,
    pub mandatory_review: MandatoryReviewId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateDisclosureResult {
    pub estate: EstateCaseId,
    pub customer: BankDisclosure<BankPrincipalId>,
    pub beneficiary: BankDisclosure<BankPrincipalId>,
    pub legal_authority: BankDisclosure<LegalAuthorityId>,
    pub account: BankDisclosure<AccountId>,
    pub posting_history: BankDisclosure<EstatePostingHistory>,
    pub audit_trail: BankDisclosure<EstateAuditTrail>,
}
