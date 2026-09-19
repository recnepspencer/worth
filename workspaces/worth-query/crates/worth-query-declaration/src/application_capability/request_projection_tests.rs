use worth_foundational::facade::AspectValue;

use crate::application_schema::{
    ApplicationFieldRef, ApplicationRelationRef, EqualityPredicate, NoApplicationUnit, ReadOnly,
};

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
struct ScopeIdentity;
struct AccountIdentity;
struct GrantAccount;

macro_rules! declare_u64_field {
    ($($field:ty),+ $(,)?) => {$ (
        impl crate::application_schema::DeclaredApplicationFieldValue for $field {
            type Value = u64;
            type Binding = crate::application_schema::U64ApplicationValueBinding;
            const PRESENCE: crate::application_schema::ApplicationFieldPresence =
                crate::application_schema::ApplicationFieldPresence::Required;
        }
    )+};
}

declare_u64_field!(ScopeIdentity, AccountIdentity);

type QueryableField<Entity, Field> = ApplicationFieldRef<
    Schema,
    Entity,
    IdentityAspect,
    Field,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

#[derive(Clone, Copy)]
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
        );
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                scope_identity(),
                crate::application_schema::ApplicationEncodedScalarValue::<
                    crate::application_schema::U64ApplicationValueBinding,
                >::try_new(*scope)
                .unwrap(),
            ),
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

fn scope_identity() -> QueryableField<Scope, ScopeIdentity> {
    ApplicationFieldRef::from_schema_identifiers("Scope", "Identity", "id")
}

fn account_identity() -> QueryableField<Account, AccountIdentity> {
    ApplicationFieldRef::from_schema_identifiers("Account", "Identity", "id")
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
    ApplicationRelationRef::from_schema_identifiers(
        "GrantAccount", "Scope", "Account",
        crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    )
}
