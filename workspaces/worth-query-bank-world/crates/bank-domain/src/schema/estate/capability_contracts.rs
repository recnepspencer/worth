use worth_query_decl::facade::worth_query_structured_value_binding;
use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context,
    worth_query_capability_context_entity_slot, worth_query_capability_provenance,
    worth_query_operation,
};

use crate::estate::EstateAction;
use crate::schema::{
    AccountActivityEffect, BankSchema, BusinessAccount, EmergencyAccess, InstitutionAccount,
    JournalEntry, JournalPosting, LegalAuthority, MandatoryReview, PersonalOwner, Posting,
    PostingAccount,
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

worth_query_operation!(pub NotifyDeathEstateOperation for BankSchema, input EstateActionInputBinding;
    reads [EstateDeathNotice, DeathNoticeSubject, EstateDeceased];
    emits [EstateDeathNotificationEffect];
);

worth_query_operation!(pub RetransmitDeathNoticeEstateOperation for BankSchema, input EstateActionInputBinding;
    reads [EstateDeathNotice, DeathNoticeSubject, EstateDeceased];
    emits [EstateDeathNotificationEffect];
);

worth_query_operation!(pub FreezeEstateAccountOperation for BankSchema, input EstateActionInputBinding;
    reads [EstateAccount];
);

worth_query_operation!(pub OpenEstateCaseOperation for BankSchema, input EstateActionInputBinding;
    reads [EstateDeathNotice];
);

worth_query_operation!(pub RecognizeEstateExecutorOperation for BankSchema, input EstateActionInputBinding;
    reads [LegalAuthorityEstate, LegalAuthorityHolder, EstateExecutor];
    links [EstateExecutor];
);

worth_query_operation!(pub DelegateEstateCapabilityOperation for BankSchema, input EstateActionInputBinding;
    reads [EstateBranch, BranchInstitution, EstateAccount];
);

// These typed read bounds mirror native-derived revocation meaning; no schema read is authored.
worth_query_operation!(pub RevokeEstateCapabilityOperation for BankSchema, input EstateActionInputBinding;
    reads [CapabilityEstate];
);

worth_query_operation!(pub RequestEstateEmergencyAccessOperation for BankSchema, input EstateActionInputBinding;
    creates [EmergencyAccess, MandatoryReview];
    links [EmergencyRequester, EmergencyGrant, EmergencyEstate, EmergencyReview, ReviewEstate];
    emits [EstateEmergencyAccessActivityEffect];
);

worth_query_operation!(pub ApproveEstateEmergencyAccessOperation for BankSchema, input EstateActionInputBinding;
    reads [EmergencyRequester, EmergencyApprover, EmergencyGrant, EmergencyEstate, EmergencyReview, ReviewEstate, ReviewPrincipal];
    links [EmergencyApprover];
    emits [EstateEmergencyAccessActivityEffect];
);

worth_query_operation!(pub RevokeEstateEmergencyAccessOperation for BankSchema, input EstateActionInputBinding;
    reads [EmergencyRequester, EmergencyApprover, EmergencyGrant, EmergencyEstate, EmergencyReview, ReviewEstate, ReviewPrincipal];
    emits [EstateEmergencyAccessActivityEffect];
);

worth_query_operation!(pub CompleteEstateMandatoryReviewOperation for BankSchema, input EstateActionInputBinding;
    reads [EmergencyRequester, EmergencyApprover, EmergencyGrant, EmergencyEstate, EmergencyReview, ReviewEstate, ReviewPrincipal];
    links [ReviewPrincipal];
    emits [EstateEmergencyAccessActivityEffect];
);

worth_query_operation!(pub ReleaseEstateOperation for BankSchema, input EstateActionInputBinding;
    reads [EstateExecutor, LegalAuthorityEstate, LegalAuthorityHolder, ReviewEstate, ReviewPrincipal];
);

worth_query_operation!(pub DisburseEstateOperation for BankSchema, input EstateActionInputBinding;
    reads [EstateAccount, EstateBeneficiary, EstateJointOwner, LegalAuthorityEstate, LegalAuthorityHolder, EstateExecutor, InstitutionAccount, PersonalOwner, BusinessAccount, PostingAccount];
    creates [JournalEntry, Posting];
    links [JournalPosting, PostingAccount];
    emits [AccountActivityEffect];
);
worth_query_operation!(pub ViewRestrictedEstateOperation for BankSchema, input EstateActionInputBinding);

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
