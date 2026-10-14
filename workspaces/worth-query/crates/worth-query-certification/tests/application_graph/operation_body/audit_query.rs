//! Independent governed readback of the created audit entity.
use super::query::{
    DocumentQueryParameters, DocumentQueryParametersBinding, DocumentRetentionRow,
    DocumentRetentionRowBinding,
};
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
worth_query_structured_value_binding!(pub AuditReadBinding for AuditRead {
    identity: "worth.query.body-court.audit-read.v1"
});

pub struct AuditIdentitySlot;
pub struct AuditRetentionSlot;
worth_query_portable_type!(AuditIdentitySlot => "worth.query.body-court.audit-identity-slot.v1");
worth_query_portable_type!(AuditRetentionSlot => "worth.query.body-court.audit-retention-slot.v1");
worth_query_application_query!(pub AuditQuery for Schema: BodySchemaBinding,
    identity "AuditQuery", parameters DocumentQueryParametersBinding,
    result DocumentRetentionRowBinding, scope Audit => "Audit", name "body_audit_read");

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

pub fn audit_query_definition<const DESTINATION: u64>() -> ApplicationQueryDefinition<
    DocumentRetentionSchema<DESTINATION>,
    AuditQuery,
    DocumentQueryParameters,
    DocumentRetentionRow,
    Audit,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema<DESTINATION>,
        AuditQuery,
        Audit,
        DocumentRetentionRow,
        DocumentRetentionRowBinding,
    >::new(Audit::reference())
    .field(identity_result())
    .field(retention_result())
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
    WorthQueryApplicationProjection<DocumentRetentionSchema<DESTINATION>, AuditQuery>
    for DocumentRetentionRow
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
    type ResultBinding = DocumentRetentionRowBinding;
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
