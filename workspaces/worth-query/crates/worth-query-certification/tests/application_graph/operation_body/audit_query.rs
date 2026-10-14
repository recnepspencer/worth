//! Independent governed readback of the created audit entity.
use super::query::{DocumentQueryParameters, DocumentQueryParametersBinding};
use super::schema::*;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};

#[derive(Clone, Debug)]
pub struct AuditRead {
    pub identity: String,
}
#[derive(Clone, Debug)]
pub struct AuditRow {
    pub identity: String,
    pub retention_days: u64,
    pub prior_retention_days: u64,
}
worth_query_portable_type!(AuditRow => "worth.query.body-court.audit-row.v1");
worth_query_structured_value_binding!(pub AuditReadBinding for AuditRead {
    identity: "worth.query.body-court.audit-read.v1"
});
worth_query_structured_value_binding!(pub AuditRowBinding for AuditRow {
    identity: "worth.query.body-court.audit-row.v1"
});

pub struct AuditIdentitySlot;
pub struct AuditRetentionSlot;
pub struct AuditPriorRetentionSlot;
worth_query_portable_type!(AuditIdentitySlot => "worth.query.body-court.audit-identity-slot.v1");
worth_query_portable_type!(AuditRetentionSlot => "worth.query.body-court.audit-retention-slot.v1");
worth_query_portable_type!(AuditPriorRetentionSlot => "worth.query.body-court.audit-prior-retention-slot.v1");
worth_query_application_query!(pub AuditQuery for Schema: BodySchemaBinding,
    identity "AuditQuery", parameters DocumentQueryParametersBinding,
    result AuditRowBinding, scope Audit => "Audit", name "body_audit_read");

fn identity_result<Schema: BodySchemaBinding>() -> ApplicationQueryResultFieldRef<
    AuditQuery,
    AuditIdentitySlot,
    Schema,
    Audit,
    AuditFacts,
    AuditIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", AuditIdentityField::reference())
}
fn retention_result<Schema: BodySchemaBinding>() -> ApplicationQueryResultFieldRef<
    AuditQuery,
    AuditRetentionSlot,
    Schema,
    Audit,
    AuditFacts,
    AuditRetentionField,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("retention", AuditRetentionField::reference())
}
fn prior_retention_result<Schema: BodySchemaBinding>() -> ApplicationQueryResultFieldRef<
    AuditQuery,
    AuditPriorRetentionSlot,
    Schema,
    Audit,
    AuditFacts,
    AuditPriorRetentionField,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("prior_retention", AuditPriorRetentionField::reference())
}

pub fn audit_query_definition<const DESTINATION: u64>() -> ApplicationQueryDefinition<
    DocumentRetentionSchema<DESTINATION>,
    AuditQuery,
    DocumentQueryParameters,
    AuditRow,
    Audit,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema<DESTINATION>,
        AuditQuery,
        Audit,
        AuditRow,
        AuditRowBinding,
    >::new(Audit::reference())
    .field(identity_result())
    .field(retention_result())
    .field(prior_retention_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(AuditQuery::reference())
        .root(Audit::reference())
        .scope(Audit::reference())
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

impl<const DESTINATION: u64>
    WorthQueryApplicationProjection<DocumentRetentionSchema<DESTINATION>, AuditQuery> for AuditRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            DocumentRetentionSchema<DESTINATION>,
            AuditQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(identity_result())?,
            retention_days: row.field(retention_result())?,
            prior_retention_days: row.field(prior_retention_result())?,
        })
    }
}

pub struct AuditQueryBinding<const DESTINATION: u64>;
type Scope<const DESTINATION: u64> = ApplicationQueryFieldScope<
    DocumentRetentionSchema<DESTINATION>,
    Audit,
    AuditFacts,
    AuditIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl<const DESTINATION: u64> ApplicationQueryBinding<DocumentRetentionSchema<DESTINATION>>
    for AuditQueryBinding<DESTINATION>
{
    type Input = AuditRead;
    type InputBinding = AuditReadBinding;
    type Query = AuditQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = AuditRowBinding;
    type ScopeBinding = Scope<DESTINATION>;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.body-court.audit-query-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<DESTINATION>,
        Audit,
        AuditFacts,
        AuditIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        AuditIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema<DESTINATION>,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl<const DESTINATION: u64> ApplicationQueryIntent<DocumentRetentionSchema<DESTINATION>>
    for AuditRead
{
    type Binding = AuditQueryBinding<DESTINATION>;
    fn parameters(&self) -> ApplicationQueryParameterSet<AuditQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope<DESTINATION> {
        Scope::new(AuditIdentityField::reference(), self.identity)
    }
}
