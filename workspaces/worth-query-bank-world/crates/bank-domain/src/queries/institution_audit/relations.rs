use worth_query_decl::facade::application_query::{
    ApplicationQueryResultRelationRef, ExactlyOneResult, ForwardResultTraversal, ManyResults,
    OptionalOneResult, ReverseResultTraversal,
};

use crate::schema::{
    Account, BankSchema, Institution, InstitutionAccount, JournalEntry, JournalPosting,
    JournalReversal, Posting, PostingAccount,
};

use super::InstitutionAuditQuery;

pub(super) struct InstitutionAccountsSlot;
worth_query_decl::facade::worth_query_portable_type!(InstitutionAccountsSlot => "InstitutionAccountsSlot");
pub(super) struct AccountPostingsSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountPostingsSlot => "AccountPostingsSlot");
pub(super) struct PostingJournalSlot;
worth_query_decl::facade::worth_query_portable_type!(PostingJournalSlot => "PostingJournalSlot");
pub(super) struct JournalReversalSlot;
worth_query_decl::facade::worth_query_portable_type!(JournalReversalSlot => "JournalReversalSlot");

pub(super) fn institution_accounts() -> ApplicationQueryResultRelationRef<
    InstitutionAuditQuery,
    InstitutionAccountsSlot,
    BankSchema,
    InstitutionAccount,
    Institution,
    Account,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("accounts", InstitutionAccount::reference())
}

pub(super) fn account_postings() -> ApplicationQueryResultRelationRef<
    InstitutionAuditQuery,
    AccountPostingsSlot,
    BankSchema,
    PostingAccount,
    Posting,
    Account,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("postings", PostingAccount::reference())
}

pub(super) fn posting_journal() -> ApplicationQueryResultRelationRef<
    InstitutionAuditQuery,
    PostingJournalSlot,
    BankSchema,
    JournalPosting,
    JournalEntry,
    Posting,
    ReverseResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("journal", JournalPosting::reference())
}

pub(super) fn journal_reversal() -> ApplicationQueryResultRelationRef<
    InstitutionAuditQuery,
    JournalReversalSlot,
    BankSchema,
    JournalReversal,
    JournalEntry,
    JournalEntry,
    ForwardResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::forward_optional("reversal_of", JournalReversal::reference())
}
