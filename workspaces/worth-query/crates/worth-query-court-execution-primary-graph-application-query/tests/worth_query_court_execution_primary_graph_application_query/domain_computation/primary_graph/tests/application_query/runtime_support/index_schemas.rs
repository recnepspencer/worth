//! Each absent equality index comes from its value type's owned binding set.
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct Bindings<const EQ: bool>;
impl<const EQ: bool> BindingSet for Bindings<EQ> {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl<const EQ: bool> Scalar<String> for Bindings<EQ> {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = EQ;
}
pub struct AccountRecords<const EQ: bool>;
impl<const EQ: bool> Fragment for AccountRecords<EQ> {
    const NAME: &'static str = "account";
    type Bindings = Bindings<EQ>;
    type Members = (Account,);
}
pub struct ActivityRecords<const EQ: bool>;
impl<const EQ: bool> Fragment for ActivityRecords<EQ> {
    const NAME: &'static str = "activity";
    type Bindings = Bindings<EQ>;
    type Members = (Activity,);
}
pub struct Account;
impl Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountPolicy,);
    type Relations = ();
}
pub struct Activity;
impl Entity for Activity {
    const NAME: &'static str = "Activity";
    type Aspects = (ActivityFacts,);
    type Relations = ();
}
pub struct AccountPolicy;
impl Aspect for AccountPolicy {
    const NAME: &'static str = "AccountPolicy";
    type Row = AccountIdentity;
}
pub struct ActivityFacts;
impl Aspect for ActivityFacts {
    const NAME: &'static str = "ActivityFacts";
    type Row = ActivityIdentity;
}
pub struct AccountIdentity(String);
pub struct ActivityIdentity(String);
impl<const EQ: bool> Record<AccountIdentity> for Bindings<EQ> {
    fn record() -> impl Shape<AccountIdentity, Self> {
        field("AccountIdentity", |row: &AccountIdentity| row.0.clone()).build(
            |key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountIdentity(key))
            },
        )
    }
}
impl<const EQ: bool> Record<ActivityIdentity> for Bindings<EQ> {
    fn record() -> impl Shape<ActivityIdentity, Self> {
        field("ActivityIdentity", |row: &ActivityIdentity| row.0.clone()).build(
            |key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ActivityIdentity(key))
            },
        )
    }
}
pub(super) mod missing_scope_index {
    use super::*;
    pub struct MissingScopeIndexSchema;
    impl Fragment for MissingScopeIndexSchema {
        const NAME: &'static str = "MissingScopeIndexSchema";
        type Bindings = NoBindings;
        type Members = (AccountRecords<false>, ActivityRecords<true>);
    }
    impl ApplicationSchema for MissingScopeIndexSchema {
        const OWNER: &'static str = "missing_scope_index_fixture";
        const NAME: &'static str = "MissingScopeIndexSchema";
        const MAJOR: u32 = 1;
        const MINOR: u32 = 0;
        fn declaration(
        ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial>
        {
            fragment::declare(Self)
        }
    }
}
pub(super) mod missing_target_index {
    use super::*;
    pub struct MissingTargetIndexSchema;
    impl Fragment for MissingTargetIndexSchema {
        const NAME: &'static str = "MissingTargetIndexSchema";
        type Bindings = NoBindings;
        type Members = (AccountRecords<true>, ActivityRecords<false>);
    }
    impl ApplicationSchema for MissingTargetIndexSchema {
        const OWNER: &'static str = "missing_target_index_fixture";
        const NAME: &'static str = "MissingTargetIndexSchema";
        const MAJOR: u32 = 1;
        const MINOR: u32 = 0;
        fn declaration(
        ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial>
        {
            fragment::declare(Self)
        }
    }
}
