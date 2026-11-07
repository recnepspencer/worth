//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct QueryRecords;
impl fragment::Fragment for QueryRecords {
    const NAME: &'static str = "query_records";
    type Bindings = QueryRecordsBindings;
    type Members = (Account, Activity);
}
pub struct QueryRecordsBindings;
impl fragment::BindingSet for QueryRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for QueryRecordsBindings {
    type Binding = worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<String> for QueryRecordsBindings {
    type Binding =
        worth_query_declaration::facade::application_schema::StringApplicationValueBinding;

    const WRITABLE: bool = false;
}
pub struct Account;
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountFacts, Identity);
    type Relations = (AccountActivity,);
}
impl Account {
    pub fn reference() -> ApplicationEntityRef<EquivalenceSchema, Self> {
        entity(QueryRecords, Self)
    }
}
pub struct Activity;
impl fragment::Entity for Activity {
    const NAME: &'static str = "Activity";
    type Aspects = (ActivityFacts,);
    type Relations = ();
}
impl Activity {
    pub fn reference() -> ApplicationEntityRef<EquivalenceSchema, Self> {
        entity(QueryRecords, Self)
    }
}
pub struct AccountFacts;
impl fragment::Aspect for AccountFacts {
    const NAME: &'static str = "AccountFacts";
    type Row = AccountFactsValue;
}

pub struct AccountFactsValue {
    pub account_id: u64,
}
impl fragment::Record<AccountFactsValue> for QueryRecordsBindings {
    fn record() -> impl fragment::Shape<AccountFactsValue, Self> {
        fragment::field("AccountId", |row: &AccountFactsValue| row.account_id)
            .equality_queryable(true)
            .build(|account_id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountFactsValue { account_id })
            })
    }
}
pub struct Identity;
impl fragment::Aspect for Identity {
    const NAME: &'static str = "Identity";
    type Row = IdentityValue;
}

pub struct IdentityValue {
    pub id: String,
}
impl fragment::Record<IdentityValue> for QueryRecordsBindings {
    fn record() -> impl fragment::Shape<IdentityValue, Self> {
        fragment::field("Id", |row: &IdentityValue| row.id.clone())
            .equality_queryable(true)
            .build(|id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IdentityValue { id })
            })
    }
}
pub struct ActivityFacts;
impl fragment::Aspect for ActivityFacts {
    const NAME: &'static str = "ActivityFacts";
    type Row = ActivityFactsValue;
}

pub struct ActivityFactsValue {
    pub activity_sequence: u64,
}
impl fragment::Record<ActivityFactsValue> for QueryRecordsBindings {
    fn record() -> impl fragment::Shape<ActivityFactsValue, Self> {
        fragment::field("ActivitySequence", |row: &ActivityFactsValue| {
            row.activity_sequence
        })
        .equality_queryable(true)
        .build(|activity_sequence, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ActivityFactsValue { activity_sequence })
        })
    }
}
pub type AccountId = FieldSelection<QueryRecords, Account, AccountFacts, u64>;
pub type Id = FieldSelection<QueryRecords, Account, Identity, String>;
pub type ActivitySequence = FieldSelection<QueryRecords, Activity, ActivityFacts, u64>;
pub struct AccountActivity;
impl fragment::Relation for AccountActivity {
    const NAME: &'static str = "AccountActivity";
    type To = Activity;
}
impl AccountActivity {
    pub fn reference() -> ApplicationRelationRef<EquivalenceSchema, Self, Account, Activity> {
        relation(QueryRecords, Account, Self)
    }
}
