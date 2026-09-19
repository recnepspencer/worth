use worth_query_decl::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_decl::facade::application_schema::{
    DeclaredApplicationUnit, EqualityPredicate, NoApplicationUnit, NoEqualityPredicate, ReadOnly,
    ReadWrite,
};

use crate::model::{
    AccountId, AccountJournalRevision, InstitutionId, JournalEntryId, PostingId, SignedMoney, USD,
};
use crate::schema::{
    Account, AccountIdentity, BankSchema, Identity, Institution, InstitutionIdentity,
    InstitutionIdentityField, JournalEntry, JournalIdentity, JournalIdentityField, JournalPurpose,
    JournalState, Posting, PostingAccountSequence, PostingAmount, PostingIdentity,
    PostingIdentityField, PostingPurpose, PostingValue, Purpose, UsdCurrency,
};

use super::InstitutionAuditQuery;

pub(super) struct InstitutionIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(InstitutionIdentitySlot => "InstitutionIdentitySlot");
pub(super) struct AccountIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AccountIdentitySlot => "AccountIdentitySlot");
pub(super) struct PostingSequenceSlot;
worth_query_decl::facade::worth_query_portable_type!(PostingSequenceSlot => "PostingSequenceSlot");
pub(super) struct PostingIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(PostingIdentitySlot => "PostingIdentitySlot");
pub(super) struct PostingAmountSlot;
worth_query_decl::facade::worth_query_portable_type!(PostingAmountSlot => "PostingAmountSlot");
pub(super) struct PostingPurposeSlot;
worth_query_decl::facade::worth_query_portable_type!(PostingPurposeSlot => "PostingPurposeSlot");
pub(super) struct JournalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(JournalIdentitySlot => "JournalIdentitySlot");
pub(super) struct JournalPurposeSlot;
worth_query_decl::facade::worth_query_portable_type!(JournalPurposeSlot => "JournalPurposeSlot");
pub(super) struct ReversalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(ReversalIdentitySlot => "ReversalIdentitySlot");

pub(super) fn institution_identity() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    InstitutionIdentitySlot,
    BankSchema,
    Institution,
    InstitutionIdentity,
    InstitutionIdentityField,
    InstitutionId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("institution", InstitutionIdentityField::reference())
}

pub(super) fn account_identity() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    AccountIdentitySlot,
    BankSchema,
    Account,
    Identity,
    AccountIdentity,
    AccountId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::reference())
}

pub(super) fn posting_sequence() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    PostingSequenceSlot,
    BankSchema,
    Posting,
    PostingValue,
    PostingAccountSequence,
    AccountJournalRevision,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("sequence", PostingAccountSequence::reference())
}

pub(super) fn posting_identity() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    PostingIdentitySlot,
    BankSchema,
    Posting,
    PostingIdentity,
    PostingIdentityField,
    PostingId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("posting", PostingIdentityField::reference())
}

pub(super) fn posting_amount() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    PostingAmountSlot,
    BankSchema,
    Posting,
    PostingValue,
    PostingAmount,
    SignedMoney<USD>,
    ReadWrite,
    NoEqualityPredicate,
    DeclaredApplicationUnit<UsdCurrency, USD>,
> {
    ApplicationQueryResultFieldRef::new("amount", PostingAmount::reference())
}

pub(super) fn posting_purpose() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    PostingPurposeSlot,
    BankSchema,
    Posting,
    PostingValue,
    Purpose,
    PostingPurpose,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("purpose", Purpose::reference())
}

pub(super) fn journal_identity<Slot: 'static>(
    name: &'static str,
) -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    Slot,
    BankSchema,
    JournalEntry,
    JournalIdentity,
    JournalIdentityField,
    JournalEntryId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new(name, JournalIdentityField::reference())
}

pub(super) fn journal_purpose() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    JournalPurposeSlot,
    BankSchema,
    JournalEntry,
    JournalState,
    JournalPurpose,
    PostingPurpose,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("purpose", JournalPurpose::reference())
}
