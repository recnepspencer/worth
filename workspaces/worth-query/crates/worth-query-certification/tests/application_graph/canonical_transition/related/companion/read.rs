//! Ordinary authenticated read projects the nested field through its binding.
use super::{super::schema::*, schema::*};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

pub struct Parameters;
pub struct ValueSlot;
pub struct IdentitySlot;
worth_query_portable_type!(IdentitySlot => "canonical-companion.first-identity.v1");
fn identity_field() -> ApplicationQueryResultFieldRef<
    ValueQuery,
    IdentitySlot,
    CanonicalSchema,
    Companion,
    CompanionFacts,
    CompanionIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", CompanionIdentityField::reference())
}
worth_query_portable_type!(ValueSlot => "canonical-companion.value-slot.v1");
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "canonical-companion.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for String { identity: "canonical-companion.result.v1" });
worth_query_application_query!(pub ValueQuery for CanonicalSchema, identity "canonical-companion.read.v1",
    parameters ParametersBinding, result ResultBinding, scope Companion => "Companion", name "companion_value");

fn value_field() -> ApplicationQueryResultFieldRef<
    ValueQuery,
    ValueSlot,
    CanonicalSchema,
    Companion,
    CompanionFacts,
    CompanionValueField,
    String,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", CompanionValueField::reference())
}

pub fn definition(
) -> ApplicationQueryDefinition<CanonicalSchema, ValueQuery, Parameters, String, Companion> {
    let shape = ApplicationQueryResultShapeBuilder::<
        CanonicalSchema,
        ValueQuery,
        Companion,
        String,
        ResultBinding,
    >::new(Companion::reference())
    .field(identity_field())
    .field(value_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ValueQuery::reference())
        .root(Companion::reference())
        .scope(Companion::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<CanonicalSchema, ValueQuery> for String {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, CanonicalSchema, ValueQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let _identity = row.field(identity_field())?;
        row.field(value_field())
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadValue {
    pub identity: String,
}
worth_query_structured_value_binding!(pub ReadInputBinding for ReadValue { identity: "canonical-companion.read-input.v1" });
pub struct ReadBinding;
type ReadScope = ApplicationQueryFieldScope<
    CanonicalSchema,
    Companion,
    CompanionFacts,
    CompanionIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<CanonicalSchema> for ReadBinding {
    type Input = ReadValue;
    type InputBinding = ReadInputBinding;
    type Query = ValueQuery;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = ReadScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "canonical-companion.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        CanonicalSchema,
        Companion,
        CompanionFacts,
        CompanionIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        CompanionIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        CanonicalSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl ApplicationQueryIntent<CanonicalSchema> for ReadValue {
    type Binding = ReadBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<ValueQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ReadScope {
        ReadScope::new(CompanionIdentityField::reference(), self.identity)
    }
}
