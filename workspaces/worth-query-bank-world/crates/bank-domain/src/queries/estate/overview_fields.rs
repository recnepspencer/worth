use worth_query_decl::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_decl::facade::application_schema::{
    EqualityPredicate, NoApplicationUnit, ReadOnly, ReadWrite,
};

use crate::estate::{
    BranchId, DeathNoticeId, DeathNoticeStatus, EstateCaseId, EstateCaseStatus,
    EstateWorkflowStage, LegalAuthorityId, LegalAuthorityKind, MandatoryReviewId,
    MandatoryReviewKind, MandatoryReviewStatus,
};
use crate::model::{AccountId, AccountName, BankPrincipalId, EmployeeAssignmentId, EmployeeRole};
use crate::schema::{
    Account, AccountDisplayName, AccountIdentity, AccountProfile, AccountState, AccountStatus,
    AssignmentRole, BankSchema, Branch, BranchIdentity, BranchIdentityField, DeathNotice,
    DeathNoticeIdentityField, DeathNoticeRecord, DeathNoticeStatusField, EmployeeAssignment,
    EmployeeAssignmentIdentityField, EmployeeScope, EstateCase, EstateCaseIdentityField,
    EstateCaseRecord, EstateCaseStatusField, EstateWorkflowStageField, Identity, LegalAuthority,
    LegalAuthorityIdentityField, LegalAuthorityKindField, LegalAuthorityRecognizedField,
    LegalAuthorityRecord, MandatoryReview, MandatoryReviewIdentityField, MandatoryReviewKindField,
    MandatoryReviewRecord, MandatoryReviewStatusField, Principal, PrincipalIdentity,
    PrincipalIdentityField, Status,
};

use super::overview::EstateCaseOverviewQuery;

pub(super) struct EstateIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(EstateIdentitySlot => "EstateIdentitySlot");
pub(super) struct EstateStageSlot;
worth_query_decl::facade::worth_query_portable_type!(EstateStageSlot => "EstateStageSlot");
pub(super) struct EstateStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(EstateStatusSlot => "EstateStatusSlot");
pub(super) struct AccountIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AccountIdentitySlot => "AccountIdentitySlot");
pub(super) struct AccountNameSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountNameSlot => "AccountNameSlot");
pub(super) struct AccountStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountStatusSlot => "AccountStatusSlot");
pub(super) struct BranchIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(BranchIdentitySlot => "BranchIdentitySlot");
pub(super) struct NoticeIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(NoticeIdentitySlot => "NoticeIdentitySlot");
pub(super) struct NoticeStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(NoticeStatusSlot => "NoticeStatusSlot");
pub(super) struct DeceasedIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(DeceasedIdentitySlot => "DeceasedIdentitySlot");
pub(super) struct ExecutorIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(ExecutorIdentitySlot => "ExecutorIdentitySlot");
pub(super) struct BeneficiaryIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(BeneficiaryIdentitySlot => "BeneficiaryIdentitySlot");
pub(super) struct AssignmentIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AssignmentIdentitySlot => "AssignmentIdentitySlot");
pub(super) struct AssignmentRoleSlot;
worth_query_decl::facade::worth_query_portable_type!(AssignmentRoleSlot => "AssignmentRoleSlot");
pub(super) struct AssignmentPrincipalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AssignmentPrincipalIdentitySlot => "AssignmentPrincipalIdentitySlot");
pub(super) struct AuthorityIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorityIdentitySlot => "AuthorityIdentitySlot");
pub(super) struct AuthorityKindSlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorityKindSlot => "AuthorityKindSlot");
pub(super) struct AuthorityRecognizedSlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorityRecognizedSlot => "AuthorityRecognizedSlot");
pub(super) struct AuthorityHolderIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorityHolderIdentitySlot => "AuthorityHolderIdentitySlot");
pub(super) struct ReviewIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewIdentitySlot => "ReviewIdentitySlot");
pub(super) struct ReviewKindSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewKindSlot => "ReviewKindSlot");
pub(super) struct ReviewStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewStatusSlot => "ReviewStatusSlot");
pub(super) struct ReviewPrincipalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewPrincipalIdentitySlot => "ReviewPrincipalIdentitySlot");

macro_rules! selector {
    (
        $name:ident,
        $slot:ty,
        $entity:ty,
        $aspect:ty,
        $field:ty,
        $value:ty,
        $write:ty,
        $alias:literal
    ) => {
        pub(super) fn $name() -> ApplicationQueryResultFieldRef<
            EstateCaseOverviewQuery,
            $slot,
            BankSchema,
            $entity,
            $aspect,
            $field,
            $value,
            $write,
            EqualityPredicate,
            NoApplicationUnit,
        > {
            ApplicationQueryResultFieldRef::new($alias, <$field>::reference())
        }
    };
}

selector!(
    estate_identity,
    EstateIdentitySlot,
    EstateCase,
    EstateCaseRecord,
    EstateCaseIdentityField,
    EstateCaseId,
    ReadOnly,
    "estate"
);
selector!(
    estate_stage,
    EstateStageSlot,
    EstateCase,
    EstateCaseRecord,
    EstateWorkflowStageField,
    EstateWorkflowStage,
    ReadWrite,
    "stage"
);
selector!(
    estate_status,
    EstateStatusSlot,
    EstateCase,
    EstateCaseRecord,
    EstateCaseStatusField,
    EstateCaseStatus,
    ReadWrite,
    "status"
);
selector!(
    account_identity,
    AccountIdentitySlot,
    Account,
    Identity,
    AccountIdentity,
    AccountId,
    ReadOnly,
    "account"
);
selector!(
    account_name,
    AccountNameSlot,
    Account,
    AccountProfile,
    AccountDisplayName,
    AccountName,
    ReadWrite,
    "display_name"
);
selector!(
    account_status,
    AccountStatusSlot,
    Account,
    AccountState,
    Status,
    AccountStatus,
    ReadWrite,
    "status"
);
selector!(
    branch_identity,
    BranchIdentitySlot,
    Branch,
    BranchIdentity,
    BranchIdentityField,
    BranchId,
    ReadOnly,
    "branch"
);
selector!(
    notice_identity,
    NoticeIdentitySlot,
    DeathNotice,
    DeathNoticeRecord,
    DeathNoticeIdentityField,
    DeathNoticeId,
    ReadOnly,
    "notice"
);
selector!(
    notice_status,
    NoticeStatusSlot,
    DeathNotice,
    DeathNoticeRecord,
    DeathNoticeStatusField,
    DeathNoticeStatus,
    ReadWrite,
    "status"
);

macro_rules! principal_selector {
    ($name:ident, $slot:ty, $alias:literal) => {
        selector!(
            $name,
            $slot,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityField,
            BankPrincipalId,
            ReadOnly,
            $alias
        );
    };
}

principal_selector!(
    deceased_identity,
    DeceasedIdentitySlot,
    "deceased_principal"
);
principal_selector!(executor_identity, ExecutorIdentitySlot, "executor");
principal_selector!(beneficiary_identity, BeneficiaryIdentitySlot, "beneficiary");
principal_selector!(
    assignment_principal_identity,
    AssignmentPrincipalIdentitySlot,
    "principal"
);
principal_selector!(
    authority_holder_identity,
    AuthorityHolderIdentitySlot,
    "holder"
);
principal_selector!(
    review_principal_identity,
    ReviewPrincipalIdentitySlot,
    "reviewer"
);

selector!(
    assignment_identity,
    AssignmentIdentitySlot,
    EmployeeAssignment,
    EmployeeScope,
    EmployeeAssignmentIdentityField,
    EmployeeAssignmentId,
    ReadOnly,
    "assignment"
);
selector!(
    assignment_role,
    AssignmentRoleSlot,
    EmployeeAssignment,
    EmployeeScope,
    AssignmentRole,
    EmployeeRole,
    ReadWrite,
    "role"
);
selector!(
    authority_identity,
    AuthorityIdentitySlot,
    LegalAuthority,
    LegalAuthorityRecord,
    LegalAuthorityIdentityField,
    LegalAuthorityId,
    ReadOnly,
    "authority"
);
selector!(
    authority_kind,
    AuthorityKindSlot,
    LegalAuthority,
    LegalAuthorityRecord,
    LegalAuthorityKindField,
    LegalAuthorityKind,
    ReadWrite,
    "kind"
);
selector!(
    authority_recognized,
    AuthorityRecognizedSlot,
    LegalAuthority,
    LegalAuthorityRecord,
    LegalAuthorityRecognizedField,
    bool,
    ReadWrite,
    "recognized"
);
selector!(
    review_identity,
    ReviewIdentitySlot,
    MandatoryReview,
    MandatoryReviewRecord,
    MandatoryReviewIdentityField,
    MandatoryReviewId,
    ReadOnly,
    "review"
);
selector!(
    review_kind,
    ReviewKindSlot,
    MandatoryReview,
    MandatoryReviewRecord,
    MandatoryReviewKindField,
    MandatoryReviewKind,
    ReadOnly,
    "kind"
);
selector!(
    review_status,
    ReviewStatusSlot,
    MandatoryReview,
    MandatoryReviewRecord,
    MandatoryReviewStatusField,
    MandatoryReviewStatus,
    ReadWrite,
    "status"
);
