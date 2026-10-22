//! Ordinary authenticated read projects the nested field through its binding.
use super::{model::NestedValue, schema::*};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

pub struct Parameters;
pub struct ValueSlot;
worth_query_portable_type!(ValueSlot => "canonical-nested.value-slot.v1");
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "canonical-nested.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for NestedValue { identity: "canonical-nested.result.v1" });
worth_query_application_query!(pub ValueQuery for CanonicalSchema, identity "canonical-nested.read.v1",
    parameters ParametersBinding, result ResultBinding, scope Document => "Document", name "nested_value");

fn value_field() -> ApplicationQueryResultFieldRef<
    ValueQuery,
    ValueSlot,
    CanonicalSchema,
    Document,
    DocumentFacts,
    DocumentValueField,
    NestedValue,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", DocumentValueField::reference())
}

pub fn definition(
) -> ApplicationQueryDefinition<CanonicalSchema, ValueQuery, Parameters, NestedValue, Document> {
    let shape = ApplicationQueryResultShapeBuilder::<
        CanonicalSchema,
        ValueQuery,
        Document,
        NestedValue,
        ResultBinding,
    >::new(Document::reference())
    .field(value_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ValueQuery::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<CanonicalSchema, ValueQuery> for NestedValue {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, CanonicalSchema, ValueQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        row.field(value_field())
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadValue {
    pub identity: String,
}
worth_query_structured_value_binding!(pub ReadInputBinding for ReadValue { identity: "canonical-nested.read-input.v1" });
pub struct ReadBinding;
type ReadScope = ApplicationQueryFieldScope<
    CanonicalSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
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
    const IDENTITY: &'static str = "canonical-nested.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        CanonicalSchema,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
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
        ReadScope::new(DocumentIdentityField::reference(), self.identity)
    }
}
