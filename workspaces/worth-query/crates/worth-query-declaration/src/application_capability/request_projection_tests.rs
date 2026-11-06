use worth_foundational::facade::AspectValue;

use crate::application_schema::ApplicationRelationRef;

use super::{
    ApplicationCapabilityContextEntitySlotRef, ApplicationCapabilityContextRef,
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRelatedEntitySelector,
    ApplicationCapabilityRequest, ApplicationCapabilityRequestContext,
    ApplicationCapabilityRequestProjection, ApplicationCapabilityRequestProjectionDenial,
};

struct Schema;
struct Capability;
struct Scope;
struct Account;
struct Context;
struct AccountSlot;
struct IdentityAspect;
struct GrantAccount;

#[derive(Clone, Copy, serde::Serialize)]
enum Input {
    Admitted {
        scope: u64,
        account: u64,
        amount: u64,
    },
    WrongVariant,
}

impl ApplicationCapabilityRequest<Schema, Capability> for Input {
    type Scope = Scope;
    type Context = Context;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let Self::Admitted {
            scope,
            account,
            amount,
        } = self
        else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "wrong input variant",
            ));
        };
        let account_selector = ApplicationCapabilityEntitySelector::new(
            account_identity(),
            crate::application_schema::ApplicationEncodedScalarValue::<
                crate::application_schema::U64ApplicationValueBinding,
            >::try_new(*account)
            .unwrap(),
        )
        .unwrap();
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                scope_identity(),
                crate::application_schema::ApplicationEncodedScalarValue::<
                    crate::application_schema::U64ApplicationValueBinding,
                >::try_new(*scope)
                .unwrap(),
            )
            .unwrap(),
            crate::application_schema::ApplicationEncodedScalarValue::<
                crate::application_schema::U64ApplicationValueBinding,
            >::try_new(7_u64)
            .unwrap(),
            crate::application_schema::ApplicationEncodedScalarValue::<
                crate::application_schema::U64ApplicationValueBinding,
            >::try_new(11_u64)
            .unwrap(),
            ApplicationCapabilityRequestContext::new(context())
                .entity(account_slot(), account_selector.clone()),
        )
        .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
            grant_account(),
            account_selector,
        ))
        .field(
            crate::application_schema::ApplicationEncodedScalarValue::<
                crate::application_schema::U64ApplicationValueBinding,
            >::try_new(13_u64)
            .unwrap(),
        )
        .magnitude(
            crate::application_schema::ApplicationEncodedScalarValue::<
                crate::application_schema::U64ApplicationValueBinding,
            >::try_new(*amount)
            .unwrap(),
        )
        .cardinality(2))
    }
}

#[test]
fn projection_retains_every_value_from_the_exact_input() {
    let input = Input::Admitted {
        scope: 17,
        account: 19,
        amount: 23,
    };
    let projection = input.capability_request().unwrap();

    assert_eq!(projection.resource().value(), &AspectValue::UInt64(17));
    assert_eq!(projection.action(), &AspectValue::UInt64(7));
    assert_eq!(projection.purpose(), &AspectValue::UInt64(11));
    assert_eq!(projection.field_value(), Some(&AspectValue::UInt64(13)));
    assert_eq!(projection.magnitude_value(), Some(&AspectValue::UInt64(23)));
    assert_eq!(projection.cardinality_value(), 2);
    assert_eq!(
        projection.related().unwrap().selector().value(),
        &AspectValue::UInt64(19)
    );
    assert_eq!(
        projection.context_value().entities()[0].selector().value(),
        &AspectValue::UInt64(19)
    );
}

#[test]
fn operation_variant_mismatch_is_explicit_input_projection_denial() {
    let denial = match Input::WrongVariant.capability_request() {
        Ok(_) => panic!("the wrong operation variant must not project"),
        Err(denial) => denial,
    };
    assert_eq!(denial.subject(), "wrong input variant");
}

fn scope_identity() -> ScopeIdentity {
    ScopeIdentity::at("id")
}

fn account_identity() -> AccountIdentity {
    AccountIdentity::at("id")
}

fn context() -> ApplicationCapabilityContextRef<Schema, Context> {
    ApplicationCapabilityContextRef::from_schema_identifier("Context")
}

fn account_slot() -> ApplicationCapabilityContextEntitySlotRef<Schema, Context, AccountSlot, Account>
{
    ApplicationCapabilityContextEntitySlotRef::from_schema_identifiers(
        context(),
        "account",
        crate::application_schema::ApplicationEntityRef::from_schema_identifier("Account"),
    )
}

fn grant_account() -> ApplicationRelationRef<Schema, GrantAccount, Scope, Account> {
    fragment::relation(QueryModel, Scope, GrantAccount)
}

use crate::application_schema::fragment;
struct QueryModel;
impl fragment::Fragment for QueryModel {
    const NAME: &'static str = "query_model";
    type Bindings = QueryBindings;
    type Members = (Scope, Account);
}
struct QueryBindings;
impl fragment::BindingSet for QueryBindings {
    type Work<'a> = crate::application_schema::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut crate::application_schema::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for QueryBindings {
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = true;
}
impl fragment::Entity for Scope {
    const NAME: &'static str = "Scope";
    type Aspects = (IdentityAspect,);
    type Relations = (GrantAccount,);
}
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (IdentityAspect,);
    type Relations = ();
}
impl fragment::Aspect for IdentityAspect {
    const NAME: &'static str = "Identity";
    type Row = QueryRow0;
}
struct QueryRow0(u64);
impl fragment::Record<QueryRow0> for QueryBindings {
    fn record() -> impl fragment::Shape<QueryRow0, Self> {
        fragment::field("id", |row: &QueryRow0| row.0).build(
            |value, _: &mut crate::application_schema::ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(QueryRow0(value))
            },
        )
    }
}
type ScopeIdentity = fragment::FieldSelection<QueryModel, Scope, IdentityAspect, u64>;
type AccountIdentity = fragment::FieldSelection<QueryModel, Account, IdentityAspect, u64>;

impl fragment::Relation for GrantAccount {
    const NAME: &'static str = "GrantAccount";
    type To = Account;
}
