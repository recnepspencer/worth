use worth_query_decl::facade::worth_query_structured_value_binding;
use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context,
    worth_query_capability_context_entity_slot, worth_query_capability_provenance,
    worth_query_operation, worth_query_operation_creates, worth_query_operation_emits,
    worth_query_operation_links, worth_query_operation_reads, worth_query_operation_writes,
};

use crate::estate::EstateAction;
use crate::schema::{
    AccountActivityEffect, AccountDisplayName, AccountIdentity, AccountingRevision, BankSchema,
    BusinessAccount, BusinessIdentityField, EmergencyAccess, InstitutionAccount,
    InstitutionIdentityField, JournalEntry, JournalIdentityField, JournalPosting, JournalPurpose,
    Kind, LegalAuthority, LegalAuthorityRecognizedField, MandatoryReview, PersonalOwner, Posting,
    PostingAccount, PostingAccountSequence, PostingAmount, PostingIdentityField,
    PrincipalIdentityField, Purpose, Status,
};

use super::*;

worth_query_capability_context!(pub EstateActionContext in BankSchema);
worth_query_capability_context_entity_slot!(
    pub EstateLegalAuthoritySlot in BankSchema,
    EstateActionContext => LegalAuthority
);
worth_query_capability_context_entity_slot!(
    pub EstateEmergencyAccessSlot in BankSchema,
    EstateActionContext => EmergencyAccess
);
worth_query_capability_context_entity_slot!(
    pub EstateMandatoryReviewSlot in BankSchema,
    EstateActionContext => MandatoryReview
);
worth_query_capability_provenance!(pub EstateGrantChainProvenance in BankSchema);

worth_query_structured_value_binding!(pub EstateActionInputBinding for EstateAction { identity: "bank.estate.action.v1" });

worth_query_operation!(pub NotifyDeathEstateOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub RetransmitDeathNoticeEstateOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub FreezeEstateAccountOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub OpenEstateCaseOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub RecognizeEstateExecutorOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub DelegateEstateCapabilityOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub RevokeEstateCapabilityOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub RequestEstateEmergencyAccessOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub ApproveEstateEmergencyAccessOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub RevokeEstateEmergencyAccessOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub CompleteEstateMandatoryReviewOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub ReleaseEstateOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub DisburseEstateOperation for BankSchema, input EstateActionInputBinding);
worth_query_operation!(pub ViewRestrictedEstateOperation for BankSchema, input EstateActionInputBinding);

worth_query_operation_reads!(FreezeEstateAccountOperation => [AccountIdentity, Status, EstateAccount]);
worth_query_operation_writes!(FreezeEstateAccountOperation => [Status]);
worth_query_operation_reads!(NotifyDeathEstateOperation => [DeathNoticeIdentityField, DeathNoticeStatusField, PrincipalIdentityField, EstateDeathNotice, DeathNoticeSubject, EstateDeceased]);
worth_query_operation_writes!(NotifyDeathEstateOperation => [DeathNoticeStatusField]);
worth_query_operation_emits!(NotifyDeathEstateOperation => [EstateDeathNotificationEffect]);
worth_query_operation_reads!(RetransmitDeathNoticeEstateOperation => [DeathNoticeIdentityField, DeathNoticeStatusField, PrincipalIdentityField, EstateDeathNotice, DeathNoticeSubject, EstateDeceased]);
worth_query_operation_emits!(RetransmitDeathNoticeEstateOperation => [EstateDeathNotificationEffect]);
worth_query_operation_reads!(OpenEstateCaseOperation => [EstateCaseIdentityField, EstateCaseStatusField, DeathNoticeIdentityField, DeathNoticeStatusField, EstateDeathNotice]);
worth_query_operation_writes!(OpenEstateCaseOperation => [EstateCaseStatusField]);
worth_query_operation_reads!(RecognizeEstateExecutorOperation => [EstateCaseIdentityField, LegalAuthorityIdentityField, LegalAuthorityRecognizedField, PrincipalIdentityField, LegalAuthorityEstate, LegalAuthorityHolder, EstateExecutor]);
worth_query_operation_links!(RecognizeEstateExecutorOperation => [EstateExecutor]);
worth_query_operation_reads!(DelegateEstateCapabilityOperation => [AccountIdentity, InstitutionIdentityField, BranchIdentityField, EstateBranch, BranchInstitution, EstateAccount]);
worth_query_operation_reads!(RevokeEstateCapabilityOperation => [CapabilityGrantIdentityField, CapabilityGrantStatusField, CapabilityEstate]);
worth_query_operation_writes!(RevokeEstateCapabilityOperation => [CapabilityGrantStatusField]);
worth_query_operation_reads!(RequestEstateEmergencyAccessOperation => [EstateCaseIdentityField]);
worth_query_operation_creates!(RequestEstateEmergencyAccessOperation => [EmergencyAccess, MandatoryReview]);
worth_query_operation_writes!(RequestEstateEmergencyAccessOperation => [EmergencyAccessIdentityField, EmergencyAccessReasonField, EmergencyAccessStatusField, EmergencyAccessIssuedAtField, EmergencyAccessExpiresAtField, MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewStatusField]);
worth_query_operation_links!(RequestEstateEmergencyAccessOperation => [EmergencyRequester, EmergencyGrant, EmergencyEstate, EmergencyReview, ReviewEstate]);
worth_query_operation_emits!(RequestEstateEmergencyAccessOperation => [EstateEmergencyAccessActivityEffect]);
worth_query_operation_reads!(ApproveEstateEmergencyAccessOperation => [EmergencyAccessIdentityField, EmergencyAccessReasonField, EmergencyAccessStatusField, EmergencyAccessIssuedAtField, EmergencyAccessExpiresAtField, MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewStatusField, EmergencyRequester, EmergencyApprover, EmergencyGrant, EmergencyEstate, EmergencyReview, ReviewEstate, ReviewPrincipal]);
worth_query_operation_writes!(ApproveEstateEmergencyAccessOperation => [EmergencyAccessStatusField]);
worth_query_operation_links!(ApproveEstateEmergencyAccessOperation => [EmergencyApprover]);
worth_query_operation_emits!(ApproveEstateEmergencyAccessOperation => [EstateEmergencyAccessActivityEffect]);
worth_query_operation_reads!(RevokeEstateEmergencyAccessOperation => [EmergencyAccessIdentityField, EmergencyAccessReasonField, EmergencyAccessStatusField, EmergencyAccessIssuedAtField, EmergencyAccessExpiresAtField, MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewStatusField, EmergencyRequester, EmergencyApprover, EmergencyGrant, EmergencyEstate, EmergencyReview, ReviewEstate, ReviewPrincipal]);
worth_query_operation_writes!(RevokeEstateEmergencyAccessOperation => [EmergencyAccessStatusField]);
worth_query_operation_emits!(RevokeEstateEmergencyAccessOperation => [EstateEmergencyAccessActivityEffect]);
worth_query_operation_reads!(CompleteEstateMandatoryReviewOperation => [EmergencyAccessIdentityField, EmergencyAccessReasonField, EmergencyAccessStatusField, EmergencyAccessIssuedAtField, EmergencyAccessExpiresAtField, MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewStatusField, EmergencyRequester, EmergencyApprover, EmergencyGrant, EmergencyEstate, EmergencyReview, ReviewEstate, ReviewPrincipal]);
worth_query_operation_writes!(CompleteEstateMandatoryReviewOperation => [MandatoryReviewStatusField]);
worth_query_operation_links!(CompleteEstateMandatoryReviewOperation => [ReviewPrincipal]);
worth_query_operation_emits!(CompleteEstateMandatoryReviewOperation => [EstateEmergencyAccessActivityEffect]);
worth_query_operation_reads!(ReleaseEstateOperation => [EstateCaseIdentityField, EstateCaseStatusField, PrincipalIdentityField, LegalAuthorityIdentityField, LegalAuthorityRecognizedField, MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewStatusField, EstateExecutor, LegalAuthorityEstate, LegalAuthorityHolder, ReviewEstate, ReviewPrincipal]);
worth_query_operation_writes!(ReleaseEstateOperation => [EstateCaseStatusField]);
worth_query_operation_reads!(DisburseEstateOperation => [EstateCaseIdentityField, EstateCaseStatusField, PrincipalIdentityField, LegalAuthorityIdentityField, LegalAuthorityRecognizedField, AccountIdentity, AccountingRevision, Status, Kind, AccountDisplayName, InstitutionIdentityField, BusinessIdentityField, PostingAmount, EstateAccount, EstateBeneficiary, EstateJointOwner, LegalAuthorityEstate, LegalAuthorityHolder, EstateExecutor, InstitutionAccount, PersonalOwner, BusinessAccount, PostingAccount]);
worth_query_operation_creates!(DisburseEstateOperation => [JournalEntry, Posting]);
worth_query_operation_writes!(DisburseEstateOperation => [AccountingRevision, JournalIdentityField, JournalPurpose, PostingIdentityField, PostingAmount, PostingAccountSequence, Purpose]);
worth_query_operation_links!(DisburseEstateOperation => [JournalPosting, PostingAccount]);
worth_query_operation_emits!(DisburseEstateOperation => [AccountActivityEffect]);

worth_query_capability!(pub NotifyDeathEstateCapability in BankSchema);
worth_query_capability!(pub RetransmitDeathNoticeEstateCapability in BankSchema);
worth_query_capability!(pub FreezeEstateAccountCapability in BankSchema);
worth_query_capability!(pub OpenEstateCaseCapability in BankSchema);
worth_query_capability!(pub RecognizeEstateExecutorCapability in BankSchema);
worth_query_capability!(pub DelegateEstateCapability in BankSchema);
worth_query_capability!(pub RevokeEstateCapability in BankSchema);
worth_query_capability!(pub RequestEstateEmergencyAccessCapability in BankSchema);
worth_query_capability!(pub ApproveEstateEmergencyAccessCapability in BankSchema);
worth_query_capability!(pub RevokeEstateEmergencyAccessCapability in BankSchema);
worth_query_capability!(pub CompleteEstateMandatoryReviewCapability in BankSchema);
worth_query_capability!(pub ReleaseEstateCapability in BankSchema);
worth_query_capability!(pub DisburseEstateCapability in BankSchema);
worth_query_capability!(pub ViewEstateAdministrationCapability in BankSchema);
worth_query_capability!(pub ViewEstateIdentityVerificationCapability in BankSchema);
worth_query_capability!(pub ViewEstateLegalComplianceCapability in BankSchema);
worth_query_capability!(pub ViewEstateEmergencyProtectionCapability in BankSchema);
worth_query_capability!(pub ViewEstateMandatoryReviewCapability in BankSchema);
