use worth_query_decl::facade::application_query::ApplicationQueryResultFieldRef;

use crate::schema::{
    AccountIdentity, BankSchema, InstitutionIdentityField, JournalIdentityField, JournalPurpose,
    PostingAccountSequence, PostingAmount, PostingIdentityField, Purpose,
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
    InstitutionIdentityField,
> {
    ApplicationQueryResultFieldRef::new(
        "institution",
        InstitutionIdentityField::at("InstitutionIdentityField"),
    )
}

pub(super) fn account_identity() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    AccountIdentitySlot,
    BankSchema,
    AccountIdentity,
> {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::at("AccountIdentity"))
}

pub(super) fn posting_sequence() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    PostingSequenceSlot,
    BankSchema,
    PostingAccountSequence,
> {
    ApplicationQueryResultFieldRef::new(
        "sequence",
        PostingAccountSequence::at("PostingAccountSequence"),
    )
}

pub(super) fn posting_identity() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    PostingIdentitySlot,
    BankSchema,
    PostingIdentityField,
> {
    ApplicationQueryResultFieldRef::new("posting", PostingIdentityField::at("PostingIdentityField"))
}

pub(super) fn posting_amount() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    PostingAmountSlot,
    BankSchema,
    PostingAmount,
> {
    ApplicationQueryResultFieldRef::new("amount", PostingAmount::at("PostingAmount"))
}

pub(super) fn posting_purpose(
) -> ApplicationQueryResultFieldRef<InstitutionAuditQuery, PostingPurposeSlot, BankSchema, Purpose>
{
    ApplicationQueryResultFieldRef::new("purpose", Purpose::at("Purpose"))
}

pub(super) fn journal_identity<Slot: 'static>(
    name: &'static str,
) -> ApplicationQueryResultFieldRef<InstitutionAuditQuery, Slot, BankSchema, JournalIdentityField> {
    ApplicationQueryResultFieldRef::new(name, JournalIdentityField::at("JournalIdentityField"))
}

pub(super) fn journal_purpose() -> ApplicationQueryResultFieldRef<
    InstitutionAuditQuery,
    JournalPurposeSlot,
    BankSchema,
    JournalPurpose,
> {
    ApplicationQueryResultFieldRef::new("purpose", JournalPurpose::at("JournalPurpose"))
}
