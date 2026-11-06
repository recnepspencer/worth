//! One record model owns field lowering and reusable scalar bindings.

use crate::{estate::*, model::*, schema::*};
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
use worth_query_decl::facade::authentication::*;
#[path = "records/rosters.rs"]
mod rosters;
pub use rosters::{AccountRecords, EstateRecords, PaymentRecords};
#[path = "records/bindings.rs"]
mod bindings;
pub use bindings::BankRecordsBindings;
#[path = "records/entity_members.rs"]
mod entity_members;
pub use entity_members::{
    Account, AccountAuthorization, Approval, Branch, Business, CapabilityGrant, Customer,
    DeathNotice, EmergencyAccess, EmployeeAssignment, EstateCase, ExternalPrincipalMapping,
    Institution, JournalEntry, LegalAuthority, MandatoryReview, PaymentIntent, Posting, Principal,
};
#[path = "records/account_rows.rs"]
mod account_rows;
pub use account_rows::{
    AccountProfile, AccountState, AuthorizationIdentity, AuthorizationScope, BusinessIdentity,
    EmployeeScope, Identity, InstitutionIdentity,
};
#[path = "records/payment_rows.rs"]
mod payment_rows;
pub use payment_rows::{
    JournalIdentity, JournalState, PaymentIdentity, PaymentState, PaymentValue, PostingIdentity,
    PostingValue, PostingValueValue,
};
#[path = "records/estate_rows.rs"]
mod estate_rows;
pub use estate_rows::{BranchIdentity, DeathNoticeRecord, EstateCaseRecord, LegalAuthorityRecord};
#[path = "records/capability_rows.rs"]
mod capability_rows;
pub use capability_rows::{CapabilityGrantRecord, EmergencyAccessRecord, MandatoryReviewRecord};
#[path = "records/principal_rows.rs"]
mod principal_rows;
pub use principal_rows::{ExternalPrincipalIdentity, PrincipalIdentity};
#[path = "records/selections.rs"]
mod selections;
pub use selections::{
    AccountAuthorizationIdentity, AccountDisplayName, AccountIdentity, AccountingRevision,
    AssignmentRole, AuthorizationRole, BranchIdentityField, BusinessIdentityField,
    CapabilityAmountCeilingField, CapabilityDelegationLimitField, CapabilityDisclosureField,
    CapabilityGrantIdentityField, CapabilityGrantStatusField, CapabilityOperationField,
    CapabilityPurposeField, CapabilityValidFromField, CapabilityValidThroughField,
    CapabilityWorkflowStageField, DeathNoticeIdentityField, DeathNoticeStatusField,
    EmergencyAccessClosedAtField, EmergencyAccessExpiresAtField, EmergencyAccessIdentityField,
    EmergencyAccessIssuedAtField, EmergencyAccessReasonField, EmergencyAccessStatusField,
    EmployeeAssignmentIdentityField, EstateCaseIdentityField, EstateCaseStatusField,
    EstateWorkflowStageField, ExternalIdentityKey, ExternalMappingStatus, InstitutionIdentityField,
    JournalIdentityField, JournalInstitutionContext, JournalPurpose, Kind,
    LegalAuthorityIdentityField, LegalAuthorityKindField, LegalAuthorityRecognizedField,
    MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewReviewedAtField,
    MandatoryReviewStatusField, PaymentAmount, PaymentIdentityField, PaymentStatusField,
    PostingAccountSequence, PostingAmount, PostingIdentityField, PrincipalIdentityField, Purpose,
    Status,
};
#[path = "records/account_relations.rs"]
mod account_relations;
pub use account_relations::{
    AccountAuthorizedUser, AssignmentPrincipal, AuthorizationAccount, BusinessAccount,
    BusinessOwner, ExternalPrincipal, InstitutionAccount, InstitutionCashAccount,
    InstitutionEmployee, PersonalOwner, PrincipalCustomer,
};
#[path = "records/payment_relations.rs"]
mod payment_relations;
pub use payment_relations::{
    ApprovalPrincipal, JournalPosting, JournalReversal, PaymentApproval, PaymentBusiness,
    PaymentDestination, PaymentInitiator, PaymentSource, PostingAccount,
};
#[path = "records/estate_relations.rs"]
mod estate_relations;
pub use estate_relations::{
    BranchInstitution, CapabilityAccount, CapabilityBranch, CapabilityEstate, CapabilityGrantee,
    CapabilityGrantor, CapabilityInstitution, CapabilityParent, DeathNoticeSubject,
    EmergencyApprover, EmergencyEstate, EmergencyGrant, EmergencyRequester, EmergencyReview,
    EstateAccount, EstateAssignment, EstateAuthorizedSigner, EstateBeneficiary, EstateBranch,
    EstateDeathNotice, EstateDeceased, EstateExecutor, EstateJointOwner, LegalAuthorityEstate,
    LegalAuthorityHolder, ReviewEstate, ReviewPrincipal,
};
