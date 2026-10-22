//! Ordinary authenticated read projects the nested field through its binding.
use super::{model::NestedValue, opaque::SwallowingValue, schema::*};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub value: NestedValue,
    pub counter: u64,
}
pub struct Parameters;
pub struct ValueSlot;
pub struct CounterSlot;
worth_query_portable_type!(CounterSlot => "canonical-admission.counter-slot.v1");
worth_query_portable_type!(ValueSlot => "canonical-admission.value-slot.v1");
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "canonical-admission.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "canonical-admission.result.v1" });
worth_query_application_query!(pub ValueQuery for AdmissionSchema, identity "canonical-admission.read.v1",
    parameters ParametersBinding, result ResultBinding, scope Document => "Document", name "nested_value");

fn value_field() -> ApplicationQueryResultFieldRef<
    ValueQuery,
    ValueSlot,
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentValueField,
    SwallowingValue,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", DocumentValueField::reference())
}

pub fn definition(
) -> ApplicationQueryDefinition<AdmissionSchema, ValueQuery, Parameters, Row, Document> {
    let shape = ApplicationQueryResultShapeBuilder::<
        AdmissionSchema,
        ValueQuery,
        Document,
        Row,
        ResultBinding,
    >::new(Document::reference())
    .field(value_field())
    .field(counter_field())
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
impl WorthQueryApplicationProjection<AdmissionSchema, ValueQuery> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, AdmissionSchema, ValueQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Row {
            value: row.field(value_field())?.0,
            counter: row.field(counter_field())?,
        })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadValue {
    pub identity: String,
}
worth_query_structured_value_binding!(pub ReadInputBinding for ReadValue { identity: "canonical-admission.read-input.v1" });
pub struct ReadBinding;
type ReadScope = ApplicationQueryFieldScope<
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<AdmissionSchema> for ReadBinding {
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
    const IDENTITY: &'static str = "canonical-admission.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        AdmissionSchema,
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
        AdmissionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl ApplicationQueryIntent<AdmissionSchema> for ReadValue {
    type Binding = ReadBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<ValueQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ReadScope {
        ReadScope::new(DocumentIdentityField::reference(), self.identity)
    }
}

fn counter_field() -> ApplicationQueryResultFieldRef<
    ValueQuery,
    CounterSlot,
    AdmissionSchema,
    Document,
    DocumentFacts,
    CounterField,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("counter", CounterField::reference())
}
