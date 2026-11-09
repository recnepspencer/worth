//! Fixture records and their query registrations.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};

pub(in crate::application_query) struct QueryTestSchema;
impl Fragment for QueryTestSchema {
    const NAME: &'static str = "QueryTestSchema";
    type Bindings = QueryBindings;
    type Members = (
        Account,
        Activity,
        ActivityQuery,
        shape_identity::GroupedOneQuery,
        shape_identity::GroupedTwoQuery,
    );
}
impl ApplicationSchema for QueryTestSchema {
    const OWNER: &'static str = "application_query_installation_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl QueryTestSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub(in crate::application_query) struct QueryBindings;
impl BindingSet for QueryBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl Scalar<u64> for QueryBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
pub(in crate::application_query) struct Account;
impl Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountFacts,);
    type Relations = (AccountActivity,);
}
pub(in crate::application_query) struct Activity;
impl Entity for Activity {
    const NAME: &'static str = "Activity";
    type Aspects = (ActivityFacts,);
    type Relations = ();
}
pub(in crate::application_query) struct AccountFacts;
impl Aspect for AccountFacts {
    const NAME: &'static str = "AccountFacts";
    type Row = AccountValue;
}
pub(in crate::application_query) struct ActivityFacts;
impl Aspect for ActivityFacts {
    const NAME: &'static str = "ActivityFacts";
    type Row = ActivityValue;
}
pub(in crate::application_query) struct AccountValue {
    id: u64,
}
pub(in crate::application_query) struct ActivityValue {
    sequence: u64,
    kind: u64,
    status: u64,
}
impl Record<AccountValue> for QueryBindings {
    fn record() -> impl Shape<AccountValue, Self> {
        field("AccountId", |r: &AccountValue| r.id)
            .equality_queryable(true)
            .build(|id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountValue { id })
            })
    }
}
impl Record<ActivityValue> for QueryBindings {
    fn record() -> impl Shape<ActivityValue, Self> {
        field("ActivitySequence", |r: &ActivityValue| r.sequence)
            .equality_queryable(true)
            .and("ActivityKind", |r: &ActivityValue| r.kind)
            .equality_queryable(true)
            .and("ActivityStatus", |r: &ActivityValue| r.status)
            .equality_queryable(true)
            .build(|sequence, kind, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ActivityValue {
                    sequence,
                    kind,
                    status,
                })
            })
    }
}
impl Account {
    pub fn reference() -> ApplicationEntityRef<QueryTestSchema, Self> {
        entity(QueryTestSchema, Self)
    }
}
impl Activity {
    pub fn reference() -> ApplicationEntityRef<QueryTestSchema, Self> {
        entity(QueryTestSchema, Self)
    }
}
pub(in crate::application_query) struct AccountActivity;
impl Relation for AccountActivity {
    const NAME: &'static str = "AccountActivity";
    type To = Activity;
}
impl AccountActivity {
    pub fn reference() -> ApplicationRelationRef<QueryTestSchema, Self, Account, Activity> {
        relation(QueryTestSchema, Account, Self)
    }
}
pub(in crate::application_query) type AccountId =
    FieldSelection<QueryTestSchema, Account, AccountFacts, u64>;
pub(in crate::application_query) type ActivitySequence =
    FieldSelection<QueryTestSchema, Activity, ActivityFacts, u64>;
pub(in crate::application_query) type ActivityKind =
    FieldSelection<QueryTestSchema, Activity, ActivityFacts, u64>;
pub(in crate::application_query) type ActivityStatus =
    FieldSelection<QueryTestSchema, Activity, ActivityFacts, u64>;
impl Registration<QueryTestSchema> for ActivityQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<QueryTestSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<QueryTestSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(definition(
            ApplicationQueryOrderingDirection::Descending,
            "sequence",
        )))
    }
}
impl Registration<QueryTestSchema> for shape_identity::GroupedOneQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<QueryTestSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<QueryTestSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(shape_identity::grouped_one_definition()))
    }
}
impl Registration<QueryTestSchema> for shape_identity::GroupedTwoQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<QueryTestSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<QueryTestSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(shape_identity::grouped_two_definition()))
    }
}
