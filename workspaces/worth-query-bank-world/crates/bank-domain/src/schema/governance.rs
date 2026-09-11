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
impl worth_query_decl::facade::application_schema::ApplicationRetainedEffectBinding
    for ActivityEventBinding
{
    fn retained_bytes(_: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of::<ActivityEvent>()).unwrap_or(u64::MAX)
    }
}
worth_query_effect!(pub AccountActivityEffect for BankSchema, payload ActivityEventBinding);

#[cfg(test)]
mod tests {
    use worth_query_decl::facade::application_schema::ApplicationRetainedEffectBinding;

    use super::{ActivityEvent, ActivityEventBinding};
    use crate::model::{AccountId, JournalEntryId, PostingId};

    #[test]
    fn activity_event_retained_bytes_are_exactly_fixed_width() {
        let event = ActivityEvent {
            account: AccountId::new(1).unwrap(),
            journal: JournalEntryId::new(2).unwrap(),
            posting: PostingId::from_operation([3; 32], 0),
            journal_sequence: 7,
        };

        assert_eq!(
            ActivityEventBinding::retained_bytes(&event),
            u64::try_from(std::mem::size_of::<ActivityEvent>()).unwrap()
        );
    }
}
