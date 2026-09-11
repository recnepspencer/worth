use worth_foundational::facade::InternedString;
use worth_query_decl::facade::worth_query_value_binding;

use crate::estate::*;

trait UnsignedIdentity {
    fn get(&self) -> u64;
}

fn encode_unsigned_identity<T: UnsignedIdentity>(value: &T) -> u64 {
    value.get()
}

macro_rules! unsigned_identity {
    ($($Value:ty),+ $(,)?) => {
        $(impl UnsignedIdentity for $Value {
            fn get(&self) -> u64 { <$Value>::get(*self) }
        })+
    };
}

unsigned_identity!(
    BranchId,
    CapabilityGrantId,
    DeathNoticeId,
    EmergencyAccessId,
    EstateCaseId,
    LegalAuthorityId,
    MandatoryReviewId
);

macro_rules! uint_binding {
    ($Binding:ident for $Value:ty, $identity:literal, $constructor:path) => {
        worth_query_value_binding! {
            pub $Binding for $Value {
                identity: $identity,
                scalar: UInt64,
                encode: encode_unsigned_identity,
                decode: $constructor,
            }
        }
    };
}

uint_binding!(BranchIdBinding for BranchId, "bank.estate.branch-id.v1", BranchId::new);
uint_binding!(CapabilityGrantIdBinding for CapabilityGrantId, "bank.estate.capability-grant-id.v1", CapabilityGrantId::new);
uint_binding!(DeathNoticeIdBinding for DeathNoticeId, "bank.estate.death-notice-id.v1", DeathNoticeId::new);
uint_binding!(EmergencyAccessIdBinding for EmergencyAccessId, "bank.estate.emergency-access-id.v1", EmergencyAccessId::new);
uint_binding!(EstateCaseIdBinding for EstateCaseId, "bank.estate.case-id.v1", EstateCaseId::new);
uint_binding!(LegalAuthorityIdBinding for LegalAuthorityId, "bank.estate.legal-authority-id.v1", LegalAuthorityId::new);
uint_binding!(MandatoryReviewIdBinding for MandatoryReviewId, "bank.estate.mandatory-review-id.v1", MandatoryReviewId::new);

fn encode_estate_moment(value: &EstateMoment) -> u64 {
    value.epoch_seconds()
}

fn decode_estate_moment(value: u64) -> Option<EstateMoment> {
    Some(EstateMoment::from_epoch_seconds(value))
}

worth_query_value_binding! {
    pub EstateMomentBinding for EstateMoment {
        identity: "bank.estate.moment.v1",
        scalar: UInt64,
        encode: encode_estate_moment,
        decode: decode_estate_moment,
    }
}

fn encode_delegation_limit(value: &DelegationLimit) -> u64 {
    u64::from(value.remaining())
}

fn decode_delegation_limit(value: u64) -> Option<DelegationLimit> {
    u8::try_from(value).ok().map(DelegationLimit::generations)
}

worth_query_value_binding! {
    pub DelegationLimitBinding for DelegationLimit {
        identity: "bank.estate.delegation-limit.v1",
        scalar: UInt64,
        encode: encode_delegation_limit,
        decode: decode_delegation_limit,
    }
}

trait StringCodec: Sized {
    fn encode(&self) -> InternedString;
    fn decode(value: InternedString) -> Option<Self>;
}

fn encode_string<T: StringCodec>(value: &T) -> InternedString {
    value.encode()
}

fn decode_string<T: StringCodec>(value: InternedString) -> Option<T> {
    T::decode(value)
}

macro_rules! string_enum_binding {
    ($Binding:ident for $Value:ty, $identity:literal, {$($Variant:path => $text:literal),+ $(,)?}) => {
        impl StringCodec for $Value {
            fn encode(&self) -> InternedString {
                match self { $($Variant => InternedString::from($text)),+ }
            }

            fn decode(value: InternedString) -> Option<Self> {
                let InternedString::Raw(value) = value else { return None; };
                match value.as_str() { $($text => Some($Variant),)+ _ => None }
            }
        }

        worth_query_value_binding! {
            pub $Binding for $Value {
                identity: $identity,
                scalar: String,
                encode: encode_string,
                decode: decode_string,
            }
        }
    };
}

string_enum_binding!(DeathNoticeStatusBinding for DeathNoticeStatus, "bank.estate.death-notice-status.v1", {
    DeathNoticeStatus::Reported => "reported", DeathNoticeStatus::NotificationRequested => "notification-requested", DeathNoticeStatus::Verified => "verified", DeathNoticeStatus::Rejected => "rejected",
});
string_enum_binding!(EstateCaseStatusBinding for EstateCaseStatus, "bank.estate.case-status.v1", {
    EstateCaseStatus::PendingOpening => "pending-opening", EstateCaseStatus::Open => "open", EstateCaseStatus::Released => "released", EstateCaseStatus::Closed => "closed",
});
string_enum_binding!(EstateWorkflowStageBinding for EstateWorkflowStage, "bank.estate.workflow-stage.v1", {
    EstateWorkflowStage::DeathReported => "death-reported", EstateWorkflowStage::AccountsFrozen => "accounts-frozen", EstateWorkflowStage::AuthorityReview => "authority-review", EstateWorkflowStage::Administration => "administration", EstateWorkflowStage::ReleaseReview => "release-review", EstateWorkflowStage::Released => "released",
});
string_enum_binding!(LegalAuthorityKindBinding for LegalAuthorityKind, "bank.estate.legal-authority-kind.v1", {
    LegalAuthorityKind::CourtAppointment => "court-appointment", LegalAuthorityKind::SmallEstateAffidavit => "small-estate-affidavit", LegalAuthorityKind::InstitutionalRecognition => "institutional-recognition",
});
string_enum_binding!(CapabilityGrantStatusBinding for CapabilityGrantStatus, "bank.estate.capability-grant-status.v1", {
    CapabilityGrantStatus::Active => "active", CapabilityGrantStatus::Revoked => "revoked",
});
string_enum_binding!(EmergencyAccessReasonBinding for EmergencyAccessReason, "bank.estate.emergency-access-reason.v1", {
    EmergencyAccessReason::PreventImmediateLoss => "prevent-immediate-loss", EmergencyAccessReason::ProtectVulnerableCustomer => "protect-vulnerable-customer", EmergencyAccessReason::MeetLegalDeadline => "meet-legal-deadline",
});
string_enum_binding!(EmergencyAccessStatusBinding for EmergencyAccessStatus, "bank.estate.emergency-access-status.v1", {
    EmergencyAccessStatus::Requested => "requested", EmergencyAccessStatus::Approved => "approved", EmergencyAccessStatus::Expired => "expired", EmergencyAccessStatus::Revoked => "revoked",
});
string_enum_binding!(MandatoryReviewStatusBinding for MandatoryReviewStatus, "bank.estate.mandatory-review-status.v1", {
    MandatoryReviewStatus::Required => "required", MandatoryReviewStatus::Completed => "completed",
});
string_enum_binding!(MandatoryReviewKindBinding for MandatoryReviewKind, "bank.estate.mandatory-review-kind.v1", {
    MandatoryReviewKind::EstateRelease => "estate-release", MandatoryReviewKind::EmergencyAccess => "emergency-access",
});
string_enum_binding!(RestrictedBankFieldBinding for RestrictedBankField, "bank.estate.restricted-field.v1", {
    RestrictedBankField::CustomerIdentity => "customer-identity", RestrictedBankField::BeneficiaryIdentity => "beneficiary-identity", RestrictedBankField::LegalDocument => "legal-document", RestrictedBankField::AccountDetails => "account-details", RestrictedBankField::PostingHistory => "posting-history", RestrictedBankField::AuditTrail => "audit-trail", RestrictedBankField::GovernanceMetadata => "governance-metadata", RestrictedBankField::EmergencyAccessActivity => "emergency-access-activity",
});
string_enum_binding!(EstateCapabilityPurposeBinding for EstateCapabilityPurpose, "bank.estate.capability-purpose.v1", {
    EstateCapabilityPurpose::EstateAdministration => "estate-administration", EstateCapabilityPurpose::IdentityVerification => "identity-verification", EstateCapabilityPurpose::LegalCompliance => "legal-compliance", EstateCapabilityPurpose::EmergencyProtection => "emergency-protection", EstateCapabilityPurpose::EstateDisbursement => "estate-disbursement", EstateCapabilityPurpose::MandatoryReview => "mandatory-review",
});
string_enum_binding!(EstateCapabilityOperationBinding for EstateCapabilityOperation, "bank.estate.capability-operation.v1", {
    EstateCapabilityOperation::NotifyDeath => "notify-death", EstateCapabilityOperation::RetransmitDeathNotice => "retransmit-death-notice", EstateCapabilityOperation::FreezeAccount => "freeze-account", EstateCapabilityOperation::OpenEstateCase => "open-estate-case", EstateCapabilityOperation::RecognizeExecutor => "recognize-executor", EstateCapabilityOperation::DelegateCapability => "delegate-capability", EstateCapabilityOperation::RevokeCapability => "revoke-capability", EstateCapabilityOperation::RequestEmergencyAccess => "request-emergency-access", EstateCapabilityOperation::ApproveEmergencyAccess => "approve-emergency-access", EstateCapabilityOperation::RevokeEmergencyAccess => "revoke-emergency-access", EstateCapabilityOperation::CompleteMandatoryReview => "complete-mandatory-review", EstateCapabilityOperation::ReleaseEstate => "release-estate", EstateCapabilityOperation::DisburseEstate => "disburse-estate", EstateCapabilityOperation::ViewRestrictedEstate => "view-restricted-estate",
});
