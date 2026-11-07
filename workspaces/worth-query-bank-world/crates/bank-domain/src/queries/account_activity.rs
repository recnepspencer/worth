use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryOrderingDirection, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationRef, ApplicationQueryResultShapeBuilder, ExactlyOneResult,
    ManyResults, OptionalOneResult, ReverseResultTraversal,
};
use worth_query_decl::facade::worth_query_application_query;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::authorization::ViewAccount;
use crate::model::AccountId;
use crate::reads::AccountActivityItem;
use crate::schema::{
    Account, AccountIdentity, BankSchema, JournalEntry, JournalIdentityField, JournalPosting,
    JournalPurpose, JournalReversal, Posting, PostingAccount, PostingAccountSequence,
    PostingAmount, PostingIdentityField, Purpose,
};

mod binding;
mod live_cause;
pub use binding::AccountActivityQueryBinding;
pub use live_cause::AccountActivityLiveCause;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountActivityQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountActivityRequest {
    account: AccountId,
}

impl AccountActivityRequest {
    pub const fn new(account: AccountId) -> Self {
        Self { account }
    }

    pub const fn account(self) -> AccountId {
        self.account
    }
}

pub const fn account_activity(account: AccountId) -> AccountActivityRequest {
    AccountActivityRequest::new(account)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountActivityQueryResult {
    account: AccountId,
    entries: Vec<AccountActivityItem>,
}

impl AccountActivityQueryResult {
    pub const fn account(&self) -> AccountId {
        self.account
    }

    pub fn entries(&self) -> &[AccountActivityItem] {
        &self.entries
    }
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountActivityQueryParametersBinding for AccountActivityQueryParameters { identity: "AccountActivityQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountActivityQueryResultBinding for AccountActivityQueryResult { identity: "AccountActivityQueryResult" });
worth_query_application_query!(
    pub AccountActivityQuery for BankSchema,
    identity "AccountActivityQuery",
    parameters AccountActivityQueryParametersBinding,
    result AccountActivityQueryResultBinding,
    scope Account => "Account",
    name "account_activity"
);
struct AccountIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AccountIdentitySlot => "AccountIdentitySlot");
struct AccountPostingsSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountPostingsSlot => "AccountPostingsSlot");
struct PostingSequenceSlot;
worth_query_decl::facade::worth_query_portable_type!(PostingSequenceSlot => "PostingSequenceSlot");
struct PostingIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(PostingIdentitySlot => "PostingIdentitySlot");
struct PostingAmountSlot;
worth_query_decl::facade::worth_query_portable_type!(PostingAmountSlot => "PostingAmountSlot");
struct PostingPurposeSlot;
worth_query_decl::facade::worth_query_portable_type!(PostingPurposeSlot => "PostingPurposeSlot");
struct PostingJournalSlot;
worth_query_decl::facade::worth_query_portable_type!(PostingJournalSlot => "PostingJournalSlot");
struct JournalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(JournalIdentitySlot => "JournalIdentitySlot");
struct JournalPurposeSlot;
worth_query_decl::facade::worth_query_portable_type!(JournalPurposeSlot => "JournalPurposeSlot");
struct JournalReversalSlot;
worth_query_decl::facade::worth_query_portable_type!(JournalReversalSlot => "JournalReversalSlot");
struct ReversalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(ReversalIdentitySlot => "ReversalIdentitySlot");

type AccountIdentitySelector = ApplicationQueryResultFieldRef<
    AccountActivityQuery,
    AccountIdentitySlot,
    BankSchema,
    AccountIdentity,
>;

type PostingSequenceSelector = ApplicationQueryResultFieldRef<
    AccountActivityQuery,
    PostingSequenceSlot,
    BankSchema,
    PostingAccountSequence,
>;

type PostingIdentitySelector = ApplicationQueryResultFieldRef<
    AccountActivityQuery,
    PostingIdentitySlot,
    BankSchema,
    PostingIdentityField,
>;

type PostingAmountSelector = ApplicationQueryResultFieldRef<
    AccountActivityQuery,
    PostingAmountSlot,
    BankSchema,
    PostingAmount,
>;

type PostingPurposeSelector =
    ApplicationQueryResultFieldRef<AccountActivityQuery, PostingPurposeSlot, BankSchema, Purpose>;

type JournalIdentitySelector<Slot> =
    ApplicationQueryResultFieldRef<AccountActivityQuery, Slot, BankSchema, JournalIdentityField>;

type JournalPurposeSelector = ApplicationQueryResultFieldRef<
    AccountActivityQuery,
    JournalPurposeSlot,
    BankSchema,
    JournalPurpose,
>;

pub fn account_activity_definition() -> ApplicationQueryDefinitionBuilder<
    BankSchema,
    AccountActivityQuery,
    AccountActivityQueryParameters,
    AccountActivityQueryResult,
    Account,
> {
    let reversal = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountActivityQuery,
        JournalEntry,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(JournalEntry::reference())
    .field(reversal_identity());
    let journal = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountActivityQuery,
        JournalEntry,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(JournalEntry::reference())
    .field(journal_identity())
    .field(journal_purpose())
    .relation(journal_reversal(), reversal);
    let posting = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountActivityQuery,
        Posting,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Posting::reference())
    .field(posting_identity())
    .field(posting_sequence())
    .field(posting_amount())
    .field(posting_purpose())
    .relation(posting_journal(), journal);
    let shape = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountActivityQuery,
        Account,
        AccountActivityQueryResult,
        AccountActivityQueryResultBinding,
    >::new(Account::reference())
    .field(account_identity())
    .relation(account_postings(), posting)
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountActivityQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(
            ApplicationQueryLaneEligibility::one_shot()
                .with_historical()
                .with_live(),
        )
        .requires_ability(ViewAccount::reference())
        .order_by(
            posting_sequence(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(account_postings())
        .live_by::<Posting, AccountActivityLiveCause>(account_identity(), posting_identity())
}

impl WorthQueryApplicationProjection<BankSchema, AccountActivityQuery>
    for AccountActivityQueryResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, AccountActivityQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let account = row.field(account_identity())?;
        let entries = row
            .many(account_postings())?
            .iter()
            .map(|posting| project_posting(account, &posting))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { account, entries })
    }
}

fn project_posting(
    account: AccountId,
    posting: &WorthQueryApplicationProjectionRow<'_, BankSchema, AccountActivityQuery>,
) -> Result<AccountActivityItem, WorthQueryApplicationProjectionDenial> {
    let _posting_id = posting.field(posting_identity())?;
    let sequence = posting.field(posting_sequence())?;
    let amount = posting.field(posting_amount())?;
    let purpose = posting.field(posting_purpose())?;
    let journal = posting.one(posting_journal())?;
    let journal_id = journal.field(journal_identity())?;
    let journal_purpose = journal.field(journal_purpose())?;
    if journal_purpose != purpose {
        return Err(WorthQueryApplicationProjectionDenial::reject(
            "posting and journal purpose disagree",
        ));
    }
    let reversal_of = journal
        .optional(journal_reversal())?
        .map(|reversal| reversal.field(reversal_identity()))
        .transpose()?;
    Ok(AccountActivityItem::from_projection(
        account,
        sequence,
        journal_id,
        purpose,
        amount,
        reversal_of,
    ))
}

fn account_identity() -> AccountIdentitySelector {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::at("AccountIdentity"))
}

fn posting_sequence() -> PostingSequenceSelector {
    ApplicationQueryResultFieldRef::new(
        "sequence",
        PostingAccountSequence::at("PostingAccountSequence"),
    )
}

fn posting_identity() -> PostingIdentitySelector {
    ApplicationQueryResultFieldRef::new("posting", PostingIdentityField::at("PostingIdentityField"))
}

fn posting_amount() -> PostingAmountSelector {
    ApplicationQueryResultFieldRef::new("amount", PostingAmount::at("PostingAmount"))
}

fn posting_purpose() -> PostingPurposeSelector {
    ApplicationQueryResultFieldRef::new("purpose", Purpose::at("Purpose"))
}

fn journal_identity() -> JournalIdentitySelector<JournalIdentitySlot> {
    ApplicationQueryResultFieldRef::new("journal", JournalIdentityField::at("JournalIdentityField"))
}

fn reversal_identity() -> JournalIdentitySelector<ReversalIdentitySlot> {
    ApplicationQueryResultFieldRef::new(
        "reversal_of",
        JournalIdentityField::at("JournalIdentityField"),
    )
}

fn journal_purpose() -> JournalPurposeSelector {
    ApplicationQueryResultFieldRef::new("purpose", JournalPurpose::at("JournalPurpose"))
}

fn account_postings() -> ApplicationQueryResultRelationRef<
    AccountActivityQuery,
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

fn posting_journal() -> ApplicationQueryResultRelationRef<
    AccountActivityQuery,
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

fn journal_reversal() -> ApplicationQueryResultRelationRef<
    AccountActivityQuery,
    JournalReversalSlot,
    BankSchema,
    JournalReversal,
    JournalEntry,
    JournalEntry,
    worth_query_decl::facade::application_query::ForwardResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::forward_optional("reversal_of", JournalReversal::reference())
}

#[cfg(test)]
mod tests;
