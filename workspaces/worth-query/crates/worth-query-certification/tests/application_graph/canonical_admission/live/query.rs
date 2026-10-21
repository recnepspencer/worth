//! The installed live projector returns success after a genuine second-read error.
use super::super::{
    opaque::SwallowingValue,
    query::{Parameters, ParametersBinding, ResultBinding, Row},
    schema::*,
};
use super::cause::LiveCause;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

pub struct IdentitySlot;
pub struct ValueSlot;
pub struct CounterSlot;
worth_query_portable_type!(IdentitySlot => "canonical-admission.live-identity-slot.v1");
worth_query_portable_type!(CounterSlot => "canonical-admission.live-counter-slot.v1");
worth_query_portable_type!(ValueSlot => "canonical-admission.live-value-slot.v1");
worth_query_application_query!(pub LiveQuery for AdmissionSchema, identity "canonical-admission.live-read.v1",
    parameters ParametersBinding, result ResultBinding, scope Document => "Document", name "live_nested_value");

type IdentityField = ApplicationQueryResultFieldRef<
    LiveQuery,
    IdentitySlot,
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;
fn identity_field() -> IdentityField {
    ApplicationQueryResultFieldRef::new("identity", DocumentIdentityField::reference())
}
fn value_field() -> ApplicationQueryResultFieldRef<
    LiveQuery,
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
fn counter_field() -> ApplicationQueryResultFieldRef<
    LiveQuery,
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
) -> ApplicationQueryDefinition<AdmissionSchema, LiveQuery, Parameters, Row, Document> {
    let shape = ApplicationQueryResultShapeBuilder::<
        AdmissionSchema,
        LiveQuery,
        Document,
        Row,
        ResultBinding,
    >::new(Document::reference())
    .field(identity_field())
    .field(value_field())
    .field(counter_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(LiveQuery::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 6))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .live_root_by::<LiveCause, _, _, _, _, _, _, _, _>(identity_field(), identity_field())
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<AdmissionSchema, LiveQuery> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, AdmissionSchema, LiveQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let value = row.field(value_field())?.0;
        let counter = row.field(counter_field())?;
        let _ = row.field(value_field());
        if super::projector_returned() {
            // A domain error after cancellation must not enter the ordinary
            // handled-error ACK path either.
            return Err(WorthQueryApplicationProjectionDenial::reject(
                "late-live-cancellation",
            ));
        }
        Ok(Row { value, counter })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadLive {
    pub identity: String,
}
worth_query_structured_value_binding!(pub LiveInputBinding for ReadLive { identity: "canonical-admission.live-input.v1" });
pub struct LiveBinding;
type ReadScope = ApplicationQueryFieldScope<
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<AdmissionSchema> for LiveBinding {
    type Input = ReadLive;
    type InputBinding = LiveInputBinding;
    type Query = LiveQuery;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = ReadScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "canonical-admission.live-binding.v1";
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
impl ApplicationQueryIntent<AdmissionSchema> for ReadLive {
    type Binding = LiveBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<LiveQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ReadScope {
        ReadScope::new(DocumentIdentityField::reference(), self.identity)
    }
}
impl ApplicationLiveQueryIntent<AdmissionSchema> for ReadLive {
    type Target = Document;
    type LiveCause = LiveCause;
}
