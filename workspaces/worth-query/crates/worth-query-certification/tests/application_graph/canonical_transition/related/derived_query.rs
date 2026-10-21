//! Ordinary authenticated read projects the nested field through its binding.
use super::{output::DerivedValue, schema::*};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

pub struct Parameters;
pub struct ValueSlot;
pub struct IdentitySlot;
worth_query_portable_type!(IdentitySlot => "canonical-related.paired.first-identity.v1");
fn identity_field() -> ApplicationQueryResultFieldRef<
    ValueQuery,
    IdentitySlot,
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", DerivedIdentityField::reference())
}
worth_query_portable_type!(ValueSlot => "canonical-transition.derived.value-slot.v1");
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "canonical-transition.derived.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for DerivedValue { identity: "canonical-transition.derived.result.v1" });
worth_query_application_query!(pub ValueQuery for CanonicalSchema, identity "canonical-transition.derived.read.v1",
    parameters ParametersBinding, result ResultBinding, scope Derived => "Derived", name "derived_value");

fn value_field() -> ApplicationQueryResultFieldRef<
    ValueQuery,
    ValueSlot,
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedValueField,
    DerivedValue,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", DerivedValueField::reference())
}

pub fn definition(
) -> ApplicationQueryDefinition<CanonicalSchema, ValueQuery, Parameters, DerivedValue, Derived> {
    let shape = ApplicationQueryResultShapeBuilder::<
        CanonicalSchema,
        ValueQuery,
        Derived,
        DerivedValue,
        ResultBinding,
    >::new(Derived::reference())
    .field(identity_field())
    .field(value_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ValueQuery::reference())
        .root(Derived::reference())
        .scope(Derived::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 5))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<CanonicalSchema, ValueQuery> for DerivedValue {
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
worth_query_structured_value_binding!(pub ReadInputBinding for ReadValue { identity: "canonical-transition.derived.read-input.v1" });
pub struct ReadBinding;
type ReadScope = ApplicationQueryFieldScope<
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedIdentityField,
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
    const IDENTITY: &'static str = "canonical-transition.derived.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        CanonicalSchema,
        Derived,
        DerivedFacts,
        DerivedIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DerivedIdentityField::reference()
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
        ReadScope::new(DerivedIdentityField::reference(), self.identity)
    }
}
