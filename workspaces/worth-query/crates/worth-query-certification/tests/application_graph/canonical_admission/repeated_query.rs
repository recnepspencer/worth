//! A declared ordinary projector catches a second decode refusal and returns success.
use super::{
    opaque::SwallowingValue,
    query::{Parameters, ParametersBinding, ResultBinding, Row},
    schema::*,
};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

pub struct ValueSlot;
pub struct CounterSlot;
worth_query_portable_type!(CounterSlot => "canonical-admission.repeated-counter-slot.v1");
worth_query_portable_type!(ValueSlot => "canonical-admission.repeated-value-slot.v1");

worth_query_application_query!(pub RepeatedQuery for AdmissionSchema, identity "canonical-admission.repeated-read.v1",
    parameters ParametersBinding, result ResultBinding, scope Document => "Document", name "repeated_nested_value");

fn value_field() -> ApplicationQueryResultFieldRef<
    RepeatedQuery,
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
) -> ApplicationQueryDefinition<AdmissionSchema, RepeatedQuery, Parameters, Row, Document> {
    let shape = ApplicationQueryResultShapeBuilder::<
        AdmissionSchema,
        RepeatedQuery,
        Document,
        Row,
        ResultBinding,
    >::new(Document::reference())
    .field(value_field())
    .field(counter_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(RepeatedQuery::reference())
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
impl WorthQueryApplicationProjection<AdmissionSchema, RepeatedQuery> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, AdmissionSchema, RepeatedQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let counter = row.field(counter_field())?;
        let value = row.field(value_field())?.0;
        // The completion owner must override this apparent projector success.
        let _ = row.field(value_field());
        RETURNED.set(RETURNED.get() + 1);
        Ok(Row { value, counter })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadRepeated {
    pub identity: String,
}
worth_query_structured_value_binding!(pub RepeatedInputBinding for ReadRepeated { identity: "canonical-admission.repeated-read-input.v1" });
pub struct RepeatedBinding;
type ReadScope = ApplicationQueryFieldScope<
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<AdmissionSchema> for RepeatedBinding {
    type Input = ReadRepeated;
    type InputBinding = RepeatedInputBinding;
    type Query = RepeatedQuery;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = ReadScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "canonical-admission.repeated-read-binding.v1";
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
impl ApplicationQueryIntent<AdmissionSchema> for ReadRepeated {
    type Binding = RepeatedBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<RepeatedQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ReadScope {
        ReadScope::new(DocumentIdentityField::reference(), self.identity)
    }
}

fn counter_field() -> ApplicationQueryResultFieldRef<
    RepeatedQuery,
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

thread_local! { static RETURNED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
pub fn reset() {
    RETURNED.set(0);
}
pub fn returned() -> u32 {
    RETURNED.get()
}
