use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
        ApplicationQueryResultShapeBuilder, ManyResults, ReverseResultTraversal,
    },
    application_schema::{
        DeclaredApplicationUnit, EqualityPredicate, NoApplicationUnit, NoEqualityPredicate,
        ReadOnly, ReadWrite,
    },
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionRow,
};

use crate::model::{AccountId, AccountJournalRevision, AccountName, SignedMoney, USD};
use crate::reads::AccountSummary;
use crate::schema::{
    Account, AccountDisplayName, AccountIdentity, AccountKind, AccountProfile, AccountState,
    AccountStatus, AccountingRevision, BankSchema, Identity, Kind, Posting, PostingAccount,
    PostingAmount, PostingValue, Status, UsdCurrency,
};

struct AccountIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AccountIdentitySlot => "AccountIdentitySlot");
struct AccountDisplayNameSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountDisplayNameSlot => "AccountDisplayNameSlot");
struct AccountKindSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountKindSlot => "AccountKindSlot");
struct AccountStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountStatusSlot => "AccountStatusSlot");
struct AccountRevisionSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountRevisionSlot => "AccountRevisionSlot");
struct AccountPostingsSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountPostingsSlot => "AccountPostingsSlot");
struct PostingAmountSlot;
worth_query_decl::facade::worth_query_portable_type!(PostingAmountSlot => "PostingAmountSlot");

type AccountIdentitySelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    AccountIdentitySlot,
    BankSchema,
    Account,
    Identity,
    AccountIdentity,
    AccountId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

type AccountDisplayNameSelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    AccountDisplayNameSlot,
    BankSchema,
    Account,
    AccountProfile,
    AccountDisplayName,
    AccountName,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
>;

type AccountKindSelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    AccountKindSlot,
    BankSchema,
    Account,
    AccountProfile,
    Kind,
    AccountKind,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
>;

type AccountStatusSelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    AccountStatusSlot,
    BankSchema,
    Account,
    AccountState,
    Status,
    AccountStatus,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
>;

type AccountRevisionSelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    AccountRevisionSlot,
    BankSchema,
    Account,
    AccountState,
    AccountingRevision,
    AccountJournalRevision,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
>;

type PostingAmountSelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    PostingAmountSlot,
    BankSchema,
    Posting,
    PostingValue,
    PostingAmount,
    SignedMoney<USD>,
    ReadWrite,
    NoEqualityPredicate,
    DeclaredApplicationUnit<UsdCurrency, USD>,
>;

pub(super) fn account_summary_shape<Query, Result, ShapeBinding>(
) -> ApplicationQueryResultShapeBuilder<BankSchema, Query, Account, Result, ShapeBinding>
where
    Query: worth_query_decl::facade::application_query::ApplicationQueryMarkerIdentity<BankSchema>,
    ShapeBinding: worth_query_decl::facade::application_schema::ApplicationStructuredValueBinding<
        Value = Result,
    >,
{
    let posting = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        Query,
        Posting,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Posting::reference())
    .field(posting_amount());
    ApplicationQueryResultShapeBuilder::new(Account::reference())
        .field(account_identity())
        .field(account_display_name())
        .field(account_kind())
        .field(account_status())
        .field(account_revision())
        .relation(account_postings(), posting)
}

pub(super) fn project_account_summary<Query>(
    row: &WorthQueryApplicationProjectionRow<'_, BankSchema, Query>,
) -> Result<AccountSummary, WorthQueryApplicationProjectionDenial>
where
    Query: worth_query_decl::facade::application_query::ApplicationQueryMarkerIdentity<BankSchema>,
{
    let account = row.field(account_identity())?;
    let revision = row.field(account_revision())?;
    let postings = row.many(account_postings())?;
    if u64::try_from(postings.len()).ok() != Some(revision.get()) {
        return Err(WorthQueryApplicationProjectionDenial::reject(
            "account posting count disagrees with accounting revision",
        ));
    }
    let balance = postings.iter().try_fold(0_i64, |balance, posting| {
        balance
            .checked_add(posting.field(posting_amount())?.minor_units())
            .ok_or_else(|| {
                WorthQueryApplicationProjectionDenial::reject(
                    "account balance exceeds signed money range",
                )
            })
    })?;
    Ok(AccountSummary::from_projection(
        account,
        row.field(account_display_name())?,
        row.field(account_kind())?,
        row.field(account_status())?,
        revision,
        SignedMoney::from_minor(balance),
        SignedMoney::from_minor(balance),
    ))
}

fn account_identity<Query>() -> AccountIdentitySelector<Query> {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::reference())
}

fn account_display_name<Query>() -> AccountDisplayNameSelector<Query> {
    ApplicationQueryResultFieldRef::new("display_name", AccountDisplayName::reference())
}

fn account_kind<Query>() -> AccountKindSelector<Query> {
    ApplicationQueryResultFieldRef::new("kind", Kind::reference())
}

fn account_status<Query>() -> AccountStatusSelector<Query> {
    ApplicationQueryResultFieldRef::new("status", Status::reference())
}

fn account_revision<Query>() -> AccountRevisionSelector<Query> {
    ApplicationQueryResultFieldRef::new("accounting_revision", AccountingRevision::reference())
}

fn posting_amount<Query>() -> PostingAmountSelector<Query> {
    ApplicationQueryResultFieldRef::new("amount", PostingAmount::reference())
}

fn account_postings<Query>() -> ApplicationQueryResultRelationRef<
    Query,
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
