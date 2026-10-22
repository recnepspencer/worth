//! Ordinary authenticated read projects the nested field through its binding.
use super::{output::DerivedValue, schema::*};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

pub struct Parameters;
pub struct ValueSlot;
worth_query_portable_type!(ValueSlot => "canonical-related.paired.value-slot.v1");
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "canonical-related.paired.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for DerivedValue { identity: "canonical-related.paired.result.v1" });
worth_query_application_query!(pub ValueQuery for CanonicalSchema, identity "canonical-related.paired.read.v1",
    parameters ParametersBinding, result ResultBinding, scope Derived => "Derived", name "related_pair_second");

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
    .field(value_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ValueQuery::reference())
        .root(Derived::reference())
        .scope(Derived::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
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
        let value = row.field(value_field())?;
        let _ = row.field(value_field());
        if super::projection::returned() {
            return Err(WorthQueryApplicationProjectionDenial::reject(
                "related-pair-domain",
            ));
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadValue {
    pub identity: String,
}
worth_query_structured_value_binding!(pub ReadInputBinding for ReadValue { identity: "canonical-related.paired.read-input.v1" });
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
    const IDENTITY: &'static str = "canonical-related.paired.read-binding.v1";
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
