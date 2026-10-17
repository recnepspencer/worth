//! The first pair read carries the identity that admits its second read's scope.
use super::super::{opaque::SwallowingValue, query, schema::*};
use worth_query_host::facade::{
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

fn identity() -> ApplicationQueryResultFieldRef<
    FirstQuery,
    IdentitySlot,
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", DocumentIdentityField::reference())
}
fn value() -> ApplicationQueryResultFieldRef<
    FirstQuery,
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
fn counter() -> ApplicationQueryResultFieldRef<
    FirstQuery,
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

pub fn definition(
) -> ApplicationQueryDefinition<AdmissionSchema, FirstQuery, query::Parameters, query::Row, Document>
{
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
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 5))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
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
