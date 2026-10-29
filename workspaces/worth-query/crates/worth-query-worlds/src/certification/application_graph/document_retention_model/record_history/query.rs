//! Payload permission has its own authored capability and actual field masks.
pub use super::*;
pub use worth_query_decl::facade::{
    worth_query_application_query, worth_query_structured_value_binding,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug)]
pub struct Input {
    pub identity: String,
}
worth_query_portable_type!(Input => "worth.query.certification.record-history.input.v1");
pub struct Parameters {
    pub identity: String,
}
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "worth.query.certification.record-history.parameters.v1" });
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub identity: String,
    pub retention: u64,
}
worth_query_portable_type!(Row => "worth.query.certification.record-history.row.v1");
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "worth.query.certification.record-history.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "worth.query.certification.record-history.row.v1" });
worth_query_application_query!(pub Query for DocumentRetentionSchema, identity "HistorySubjectRead", parameters ParametersBinding,
    result ResultBinding, scope Document => "Document", name "history_subject_read");
pub struct IdentitySlot;
pub struct RetentionSlot;
worth_query_portable_type!(IdentitySlot => "worth.query.certification.record-history.identity-slot.v1");
worth_query_portable_type!(RetentionSlot => "worth.query.certification.record-history.retention-slot.v1");
pub fn identity() -> ApplicationQueryResultFieldRef<
    Query,
    IdentitySlot,
    DocumentRetentionSchema,
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
pub fn retention() -> ApplicationQueryResultFieldRef<
    Query,
    RetentionSlot,
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentRetentionField,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("retention", DocumentRetentionField::reference())
}

pub struct Binding;
pub type Scope = ApplicationQueryFieldScope<
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<DocumentRetentionSchema> for Binding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.certification.record-history.governed-subject.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema,
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
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

pub struct IdentityParameter;
pub fn parameter(
) -> ApplicationQueryParameterRef<Query, IdentityParameter, StringApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("identity")
}
pub fn parameters(identity: &str) -> ApplicationQueryParameterSet<Query> {
    let input = Input {
        identity: identity.to_owned(),
    };
    let parameters = Parameters {
        identity: input.identity,
    };
    ApplicationQueryParameterSet::new()
        .bind(parameter(), parameters.identity)
        .unwrap()
}
pub fn definition(
) -> ApplicationQueryDefinition<DocumentRetentionSchema, Query, Parameters, Row, Document> {
    let influence = ApplicationQueryInfluenceContract::permit_all();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "record-history-subject",
        super::capability::PayloadCapability::reference(),
    )
    .use_field_by(
        DocumentIdentityField::reference(),
        super::capability::encoded(),
        influence.clone(),
    )
    .use_field_by(
        DocumentRetentionField::reference(),
        super::capability::encoded(),
        influence.clone(),
    )
    .disclose_field_by(identity(), super::capability::encoded(), influence.clone())
    .disclose_field_by(retention(), super::capability::encoded(), influence);
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema,
        Query,
        Document,
        Row,
        ResultBinding,
    >::new(Document::reference())
    .field(identity())
    .field(retention())
    .build();
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(parameter())
        .where_equal(DocumentIdentityField::reference(), parameter())
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<DocumentRetentionSchema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DocumentRetentionSchema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(identity())?,
            retention: row.field(retention())?,
        })
    }
}
