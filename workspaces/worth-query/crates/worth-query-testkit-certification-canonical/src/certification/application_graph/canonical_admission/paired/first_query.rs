//! The first pair read carries the identity that admits its second read's scope.
pub use super::super::{opaque::SwallowingValue, query, schema::*};
pub use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

pub struct IdentitySlot;
pub struct ValueSlot;
pub struct CounterSlot;
worth_query_portable_type!(IdentitySlot => "canonical-admission.paired.first-identity.v1");
worth_query_portable_type!(ValueSlot => "canonical-admission.paired.first-value.v1");
worth_query_portable_type!(CounterSlot => "canonical-admission.paired.first-counter.v1");
worth_query_application_query!(pub FirstQuery for AdmissionSchema,
    identity "canonical-admission.paired.first.v1", parameters query::ParametersBinding,
    result query::ResultBinding, scope Document => "Document", name "paired_canonical_first");

pub fn identity(
) -> ApplicationQueryResultFieldRef<FirstQuery, IdentitySlot, AdmissionSchema, DocumentIdentityField>
{
    ApplicationQueryResultFieldRef::new(
        "identity",
        DocumentIdentityField::at("DocumentIdentityField"),
    )
}
pub fn value(
) -> ApplicationQueryResultFieldRef<FirstQuery, ValueSlot, AdmissionSchema, DocumentValueField> {
    ApplicationQueryResultFieldRef::new("value", DocumentValueField::at("DocumentValueField"))
}
pub fn counter(
) -> ApplicationQueryResultFieldRef<FirstQuery, CounterSlot, AdmissionSchema, CounterField> {
    ApplicationQueryResultFieldRef::new("counter", CounterField::at("CounterField"))
}

pub fn definition() -> ApplicationQueryDefinitionBuilder<
    AdmissionSchema,
    FirstQuery,
    query::Parameters,
    query::Row,
    Document,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        AdmissionSchema,
        FirstQuery,
        Document,
        query::Row,
        query::ResultBinding,
    >::new(Document::reference())
    .field(identity())
    .field(value())
    .field(counter())
    .build();
    ApplicationQueryDefinitionBuilder::declare(FirstQuery::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
impl WorthQueryApplicationProjection<AdmissionSchema, FirstQuery> for query::Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, AdmissionSchema, FirstQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            value: row.field(value())?.0,
            counter: row.field(counter())?,
        })
    }
}

pub struct FirstBinding;
impl ApplicationQueryBinding<AdmissionSchema> for FirstBinding {
    type Input = query::ReadValue;
    type InputBinding = query::ReadInputBinding;
    type Query = FirstQuery;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = query::ResultBinding;
    type ScopeBinding =
        <query::ReadBinding as ApplicationQueryBinding<AdmissionSchema>>::ScopeBinding;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "canonical-admission.paired.first-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> DocumentIdentityField {
        query::ReadBinding::scope_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        AdmissionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        query::ReadBinding::principal_binding()
    }
}
