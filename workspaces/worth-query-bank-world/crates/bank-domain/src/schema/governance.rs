use worth_query_decl::facade::worth_query_structured_value_binding;
use worth_query_decl::facade::{worth_query_effect, worth_query_policy, worth_query_unit};

use crate::model::{AccountId, JournalEntryId, PostingId, USD};

use super::BankSchema;

worth_query_policy!(pub AccountVisibilityPolicy in BankSchema);
worth_query_policy!(pub AccountMutationScopePolicy in BankSchema);
worth_query_policy!(pub EmployeeScopePolicy in BankSchema);
worth_query_policy!(pub DistinctApproverPolicy in BankSchema);
worth_query_unit!(pub UsdCurrency(USD) in BankSchema);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityEvent {
    pub account: AccountId,
    pub journal: JournalEntryId,
    pub posting: PostingId,
    pub journal_sequence: u64,
}

worth_query_structured_value_binding!(pub ActivityEventBinding for ActivityEvent { identity: "bank.effect.account-activity.payload.v1" });

worth_query_effect!(pub AccountActivityEffect for BankSchema, payload ActivityEventBinding);
