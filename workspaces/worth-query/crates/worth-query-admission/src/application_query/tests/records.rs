//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct PlanningModel;
impl fragment::Fragment for PlanningModel {
    const NAME: &'static str = "planning_model";
    type Bindings = PlanningModelBindings;
    type Members = (Account, Activity);
}
pub struct PlanningModelBindings;
impl fragment::BindingSet for PlanningModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for PlanningModelBindings {
    type Binding = worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
pub struct Account;
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountFacts,);
    type Relations = (AccountActivity, AccountNext);
}
impl Account {
    pub fn reference() -> ApplicationEntityRef<PlanningTestSchema, Self> {
        entity(PlanningModel, Self)
    }
}
pub struct Activity;
impl fragment::Entity for Activity {
    const NAME: &'static str = "Activity";
    type Aspects = (ActivityFacts,);
    type Relations = ();
}
impl Activity {
    pub fn reference() -> ApplicationEntityRef<PlanningTestSchema, Self> {
        entity(PlanningModel, Self)
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
impl fragment::Record<AccountFactsValue> for PlanningModelBindings {
    fn record() -> impl fragment::Shape<AccountFactsValue, Self> {
        fragment::field("AccountId", |row: &AccountFactsValue| row.account_id).build(
            |account_id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountFactsValue { account_id })
            },
        )
    }
}
pub struct ActivityFacts;
impl fragment::Aspect for ActivityFacts {
    const NAME: &'static str = "ActivityFacts";
    type Row = ActivityFactsValue;
}

pub struct ActivityFactsValue {
    pub sequence: u64,
}
impl fragment::Record<ActivityFactsValue> for PlanningModelBindings {
    fn record() -> impl fragment::Shape<ActivityFactsValue, Self> {
        fragment::field("Sequence", |row: &ActivityFactsValue| row.sequence).build(
            |sequence, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ActivityFactsValue { sequence })
            },
        )
    }
}
pub type AccountId = FieldSelection<PlanningModel, Account, AccountFacts, u64>;
pub type Sequence = FieldSelection<PlanningModel, Activity, ActivityFacts, u64>;
pub struct AccountActivity;
impl fragment::Relation for AccountActivity {
    const NAME: &'static str = "AccountActivity";
    type To = Activity;
}
impl AccountActivity {
    pub fn reference() -> ApplicationRelationRef<PlanningTestSchema, Self, Account, Activity> {
        relation(PlanningModel, Account, Self)
    }
}
pub struct AccountNext;
impl fragment::Relation for AccountNext {
    const NAME: &'static str = "AccountNext";
    type To = Account;
}
impl AccountNext {
    pub fn reference() -> ApplicationRelationRef<PlanningTestSchema, Self, Account, Account> {
        relation(PlanningModel, Account, Self)
    }
}
