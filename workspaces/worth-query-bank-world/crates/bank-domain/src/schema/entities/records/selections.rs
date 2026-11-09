//! Selections of the bank model.
use super::*;
pub type AccountIdentity = FieldSelection<AccountRecords, Account, Identity, AccountId>;
pub type InstitutionIdentityField =
    FieldSelection<AccountRecords, Institution, InstitutionIdentity, InstitutionId>;
pub type BusinessIdentityField =
    FieldSelection<AccountRecords, Business, BusinessIdentity, BusinessId>;
pub type PaymentIdentityField =
    FieldSelection<PaymentRecords, PaymentIntent, PaymentIdentity, PaymentId>;
pub type AccountDisplayName = FieldSelection<AccountRecords, Account, AccountProfile, AccountName>;
pub type Kind = FieldSelection<AccountRecords, Account, AccountProfile, AccountKind>;
pub type AccountingRevision =
    FieldSelection<AccountRecords, Account, AccountState, AccountJournalRevision>;
pub type Status = FieldSelection<AccountRecords, Account, AccountState, AccountStatus>;
pub type AuthorizationRole =
    FieldSelection<AccountRecords, AccountAuthorization, AuthorizationScope, CustomerRole>;
pub type AccountAuthorizationIdentity = FieldSelection<
    AccountRecords,
    AccountAuthorization,
    AuthorizationIdentity,
    AccountAuthorizationId,
>;
pub type EmployeeAssignmentIdentityField =
    FieldSelection<AccountRecords, EmployeeAssignment, EmployeeScope, EmployeeAssignmentId>;
pub type AssignmentRole =
    FieldSelection<AccountRecords, EmployeeAssignment, EmployeeScope, EmployeeRole>;
pub type PostingAmount = FieldSelection<PaymentRecords, Posting, PostingValue, SignedMoney<USD>>;
pub type PostingAccountSequence =
    FieldSelection<PaymentRecords, Posting, PostingValue, AccountJournalRevision>;
pub type PostingIdentityField = FieldSelection<PaymentRecords, Posting, PostingIdentity, PostingId>;
pub type Purpose = FieldSelection<PaymentRecords, Posting, PostingValue, PostingPurpose>;
pub type PaymentStatusField =
    FieldSelection<PaymentRecords, PaymentIntent, PaymentState, PaymentStatus>;
pub type JournalIdentityField =
    FieldSelection<PaymentRecords, JournalEntry, JournalIdentity, JournalEntryId>;
pub type JournalInstitutionContext =
    FieldSelection<PaymentRecords, JournalEntry, JournalIdentity, InstitutionId>;
pub type JournalPurpose =
    FieldSelection<PaymentRecords, JournalEntry, JournalState, PostingPurpose>;
pub type PaymentAmount = FieldSelection<PaymentRecords, PaymentIntent, PaymentValue, Money<USD>>;
pub type BranchIdentityField = FieldSelection<EstateRecords, Branch, BranchIdentity, BranchId>;
pub type DeathNoticeIdentityField =
    FieldSelection<EstateRecords, DeathNotice, DeathNoticeRecord, DeathNoticeId>;
pub type DeathNoticeStatusField =
    FieldSelection<EstateRecords, DeathNotice, DeathNoticeRecord, DeathNoticeStatus>;
pub type EstateCaseIdentityField =
    FieldSelection<EstateRecords, EstateCase, EstateCaseRecord, EstateCaseId>;
pub type EstateWorkflowStageField =
    FieldSelection<EstateRecords, EstateCase, EstateCaseRecord, EstateWorkflowStage>;
pub type EstateCaseStatusField =
    FieldSelection<EstateRecords, EstateCase, EstateCaseRecord, EstateCaseStatus>;
pub type LegalAuthorityIdentityField =
    FieldSelection<EstateRecords, LegalAuthority, LegalAuthorityRecord, LegalAuthorityId>;
pub type LegalAuthorityKindField =
    FieldSelection<EstateRecords, LegalAuthority, LegalAuthorityRecord, LegalAuthorityKind>;
pub type LegalAuthorityRecognizedField =
    FieldSelection<EstateRecords, LegalAuthority, LegalAuthorityRecord, bool>;
pub type CapabilityGrantIdentityField =
    FieldSelection<EstateRecords, CapabilityGrant, CapabilityGrantRecord, CapabilityGrantId>;
pub type CapabilityOperationField = FieldSelection<
    EstateRecords,
    CapabilityGrant,
    CapabilityGrantRecord,
    EstateCapabilityOperation,
>;
pub type CapabilityPurposeField =
    FieldSelection<EstateRecords, CapabilityGrant, CapabilityGrantRecord, EstateCapabilityPurpose>;
pub type CapabilityDisclosureField =
    FieldSelection<EstateRecords, CapabilityGrant, CapabilityGrantRecord, RestrictedBankField>;
pub type CapabilityAmountCeilingField =
    FieldSelection<EstateRecords, CapabilityGrant, CapabilityGrantRecord, Money<USD>>;
pub type CapabilityValidFromField =
    FieldSelection<EstateRecords, CapabilityGrant, CapabilityGrantRecord, EstateMoment>;
pub type CapabilityValidThroughField =
    FieldSelection<EstateRecords, CapabilityGrant, CapabilityGrantRecord, EstateMoment>;
pub type CapabilityDelegationLimitField =
    FieldSelection<EstateRecords, CapabilityGrant, CapabilityGrantRecord, DelegationLimit>;
pub type CapabilityWorkflowStageField =
    FieldSelection<EstateRecords, CapabilityGrant, CapabilityGrantRecord, EstateWorkflowStage>;
pub type CapabilityGrantStatusField =
    FieldSelection<EstateRecords, CapabilityGrant, CapabilityGrantRecord, CapabilityGrantStatus>;
pub type EmergencyAccessIdentityField =
    FieldSelection<EstateRecords, EmergencyAccess, EmergencyAccessRecord, EmergencyAccessId>;
pub type EmergencyAccessReasonField =
    FieldSelection<EstateRecords, EmergencyAccess, EmergencyAccessRecord, EmergencyAccessReason>;
pub type EmergencyAccessStatusField =
    FieldSelection<EstateRecords, EmergencyAccess, EmergencyAccessRecord, EmergencyAccessStatus>;
pub type EmergencyAccessIssuedAtField =
    FieldSelection<EstateRecords, EmergencyAccess, EmergencyAccessRecord, EstateMoment>;
pub type EmergencyAccessExpiresAtField =
    FieldSelection<EstateRecords, EmergencyAccess, EmergencyAccessRecord, EstateMoment>;
pub type EmergencyAccessClosedAtField =
    FieldSelection<EstateRecords, EmergencyAccess, EmergencyAccessRecord, EstateMoment>;
pub type MandatoryReviewIdentityField =
    FieldSelection<EstateRecords, MandatoryReview, MandatoryReviewRecord, MandatoryReviewId>;
pub type MandatoryReviewStatusField =
    FieldSelection<EstateRecords, MandatoryReview, MandatoryReviewRecord, MandatoryReviewStatus>;
pub type MandatoryReviewReviewedAtField =
    FieldSelection<EstateRecords, MandatoryReview, MandatoryReviewRecord, EstateMoment>;
pub type MandatoryReviewKindField =
    FieldSelection<EstateRecords, MandatoryReview, MandatoryReviewRecord, MandatoryReviewKind>;
pub type ExternalIdentityKey = FieldSelection<
    AccountRecords,
    ExternalPrincipalMapping,
    ExternalPrincipalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type ExternalMappingStatus = FieldSelection<
    AccountRecords,
    ExternalPrincipalMapping,
    ExternalPrincipalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField =
    FieldSelection<AccountRecords, Principal, PrincipalIdentity, BankPrincipalId>;
