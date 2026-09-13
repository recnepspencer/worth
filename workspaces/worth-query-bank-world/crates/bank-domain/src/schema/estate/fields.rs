use worth_query_decl::facade::application_schema::BoolApplicationValueBinding;
use worth_query_decl::facade::{worth_query_aspect, worth_query_field};

use crate::estate::{
    BranchId, CapabilityGrantId, CapabilityGrantStatus, DeathNoticeId, DeathNoticeStatus,
    DelegationLimit, EmergencyAccessId, EmergencyAccessReason, EmergencyAccessStatus,
    EstateCapabilityOperation, EstateCapabilityPurpose, EstateCaseId, EstateCaseStatus,
    EstateMoment, EstateWorkflowStage, LegalAuthorityId, LegalAuthorityKind, MandatoryReviewId,
    MandatoryReviewKind, MandatoryReviewStatus, RestrictedBankField,
};
use crate::model::{Money, USD};
use crate::schema::BankSchema;
use crate::schema::UsdMoneyBinding;

use super::entities::{
    Branch, CapabilityGrant, DeathNotice, EmergencyAccess, EstateCase, LegalAuthority,
    MandatoryReview,
};
use super::values::{
    BranchIdBinding, CapabilityGrantIdBinding, CapabilityGrantStatusBinding, DeathNoticeIdBinding,
    DeathNoticeStatusBinding, DelegationLimitBinding, EmergencyAccessIdBinding,
    EmergencyAccessReasonBinding, EmergencyAccessStatusBinding, EstateCapabilityOperationBinding,
    EstateCapabilityPurposeBinding, EstateCaseIdBinding, EstateCaseStatusBinding,
    EstateMomentBinding, EstateWorkflowStageBinding, LegalAuthorityIdBinding,
    LegalAuthorityKindBinding, MandatoryReviewIdBinding, MandatoryReviewKindBinding,
    MandatoryReviewStatusBinding, RestrictedBankFieldBinding,
};

worth_query_aspect!(pub BranchIdentity for BankSchema, Branch; identity = AspectIdentity(0x91611003), revision = AspectContractRevision(1),);
worth_query_aspect!(pub DeathNoticeRecord for BankSchema, DeathNotice; identity = AspectIdentity(0x91611004), revision = AspectContractRevision(1),);
worth_query_aspect!(pub EstateCaseRecord for BankSchema, EstateCase; identity = AspectIdentity(0x91611005), revision = AspectContractRevision(1),);
worth_query_aspect!(pub LegalAuthorityRecord for BankSchema, LegalAuthority; identity = AspectIdentity(0x91611006), revision = AspectContractRevision(1),);
worth_query_aspect!(pub CapabilityGrantRecord for BankSchema, CapabilityGrant; identity = AspectIdentity(0x91611007), revision = AspectContractRevision(1),);
worth_query_aspect!(pub EmergencyAccessRecord for BankSchema, EmergencyAccess; identity = AspectIdentity(0x91611008), revision = AspectContractRevision(1),);
worth_query_aspect!(pub MandatoryReviewRecord for BankSchema, MandatoryReview; identity = AspectIdentity(0x91611009), revision = AspectContractRevision(1),);

worth_query_field!(
    pub BranchIdentityField for BankSchema, Branch, BranchIdentity:
    BranchId => BranchIdBinding, read_only, equality
);
worth_query_field!(
    pub DeathNoticeIdentityField for BankSchema, DeathNotice, DeathNoticeRecord:
    DeathNoticeId => DeathNoticeIdBinding, read_only, equality
);
worth_query_field!(
    pub DeathNoticeStatusField for BankSchema, DeathNotice, DeathNoticeRecord:
    DeathNoticeStatus => DeathNoticeStatusBinding, read_write, equality
);
worth_query_field!(
    pub EstateCaseIdentityField for BankSchema, EstateCase, EstateCaseRecord:
    EstateCaseId => EstateCaseIdBinding, read_only, equality
);
worth_query_field!(
    pub EstateWorkflowStageField for BankSchema, EstateCase, EstateCaseRecord:
    EstateWorkflowStage => EstateWorkflowStageBinding, read_write, equality
);
worth_query_field!(
    pub EstateCaseStatusField for BankSchema, EstateCase, EstateCaseRecord:
    EstateCaseStatus => EstateCaseStatusBinding, read_write, equality
);
worth_query_field!(
    pub LegalAuthorityIdentityField for BankSchema, LegalAuthority, LegalAuthorityRecord:
    LegalAuthorityId => LegalAuthorityIdBinding, read_only, equality
);
worth_query_field!(
    pub LegalAuthorityKindField for BankSchema, LegalAuthority, LegalAuthorityRecord:
    LegalAuthorityKind => LegalAuthorityKindBinding, read_write, equality
);
worth_query_field!(
    pub LegalAuthorityRecognizedField for BankSchema, LegalAuthority, LegalAuthorityRecord:
    bool => BoolApplicationValueBinding, read_write, equality
);
worth_query_field!(
    pub CapabilityGrantIdentityField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    CapabilityGrantId => CapabilityGrantIdBinding, read_only, equality
);
worth_query_field!(
    pub CapabilityOperationField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    EstateCapabilityOperation => EstateCapabilityOperationBinding, read_write, equality
);
worth_query_field!(
    pub CapabilityPurposeField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    EstateCapabilityPurpose => EstateCapabilityPurposeBinding, read_write, equality
);
worth_query_field!(
    pub CapabilityDisclosureField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    optional RestrictedBankField => RestrictedBankFieldBinding, read_write, equality
);
worth_query_field!(
    pub CapabilityAmountCeilingField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    optional Money<USD> => UsdMoneyBinding, unit crate::schema::UsdCurrency, read_write, no_equality
);
worth_query_field!(
    pub CapabilityValidFromField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    EstateMoment => EstateMomentBinding, read_write, equality
);
worth_query_field!(
    pub CapabilityValidThroughField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    EstateMoment => EstateMomentBinding, read_write, equality
);
worth_query_field!(
    pub CapabilityDelegationLimitField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    DelegationLimit => DelegationLimitBinding, read_write, equality
);
worth_query_field!(
    pub CapabilityWorkflowStageField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    EstateWorkflowStage => EstateWorkflowStageBinding, read_write, equality
);
worth_query_field!(
    pub CapabilityGrantStatusField for BankSchema, CapabilityGrant, CapabilityGrantRecord:
    CapabilityGrantStatus => CapabilityGrantStatusBinding, read_write, equality
);
worth_query_field!(
    pub EmergencyAccessIdentityField for BankSchema, EmergencyAccess, EmergencyAccessRecord:
    EmergencyAccessId => EmergencyAccessIdBinding, read_only, equality
);
worth_query_field!(
    pub EmergencyAccessReasonField for BankSchema, EmergencyAccess, EmergencyAccessRecord:
    EmergencyAccessReason => EmergencyAccessReasonBinding, read_write, equality
);
worth_query_field!(
    pub EmergencyAccessStatusField for BankSchema, EmergencyAccess, EmergencyAccessRecord:
    EmergencyAccessStatus => EmergencyAccessStatusBinding, read_write, equality
);
worth_query_field!(
    pub EmergencyAccessIssuedAtField for BankSchema, EmergencyAccess, EmergencyAccessRecord:
    EstateMoment => EstateMomentBinding, read_write, equality
);
worth_query_field!(
    pub EmergencyAccessExpiresAtField for BankSchema, EmergencyAccess, EmergencyAccessRecord:
    EstateMoment => EstateMomentBinding, read_write, equality
);
worth_query_field!(
    pub MandatoryReviewIdentityField for BankSchema, MandatoryReview, MandatoryReviewRecord:
    MandatoryReviewId => MandatoryReviewIdBinding, read_only, equality
);
worth_query_field!(
    pub MandatoryReviewStatusField for BankSchema, MandatoryReview, MandatoryReviewRecord:
    MandatoryReviewStatus => MandatoryReviewStatusBinding, read_write, equality
);
worth_query_field!(
    pub MandatoryReviewKindField for BankSchema, MandatoryReview, MandatoryReviewRecord:
    MandatoryReviewKind => MandatoryReviewKindBinding, read_only, equality
);
