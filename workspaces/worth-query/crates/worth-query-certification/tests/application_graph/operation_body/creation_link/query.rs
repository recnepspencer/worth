//! Independent native traversal sees the committed created-to-scope endpoint.
use super::super::{
    genesis::declaration::{DocumentTenant, Tenant, TenantFacts, TenantIdentity},
    query::DocumentQueryParametersBinding,
    schema::*,
};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRow(pub u64);
worth_query_structured_value_binding!(pub TenantRowBinding for TenantRow { identity: "worth.query.body-court.link-row.v1" });
#[derive(Clone)]
pub struct ReadTenant(pub String);
worth_query_structured_value_binding!(pub ReadTenantBinding for ReadTenant { identity: "worth.query.body-court.link-input.v1" });
pub struct TenantSlot;
worth_query_portable_type!(TenantSlot => "worth.query.body-court.link-tenant-slot.v1");
worth_query_application_query!(pub TenantQuery for Schema: BodySchemaBinding,
    identity "TenantQuery", parameters DocumentQueryParametersBinding, result TenantRowBinding,
    scope Document => "Document", name "body_link_tenant");
fn field<Schema: BodySchemaBinding>() -> ApplicationQueryResultFieldRef<
    TenantQuery,
    TenantSlot,
    Schema,
    Tenant,
    TenantFacts,
    TenantIdentity,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("tenant", TenantIdentity::reference())
}
pub(in super::super) fn declare<const MODE: u64>(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<MODE>>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<MODE>> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema<MODE>,
        TenantQuery,
        Tenant,
        TenantRow,
        TenantRowBinding,
    >::new(Tenant::reference())
    .field(field())
    .build();
    let query = ApplicationQueryDefinitionBuilder::declare(TenantQuery::reference())
        .root(Tenant::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 8))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Document::reference())
                .forward(DocumentTenant::reference()),
        )
        .build()
        .unwrap();
    schema
        .application_query(query)
        .application_query_binding::<TenantBinding<MODE>>()
}
impl<const MODE: u64> WorthQueryApplicationProjection<DocumentRetentionSchema<MODE>, TenantQuery>
    for TenantRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DocumentRetentionSchema<MODE>, TenantQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(field())?))
    }
}
pub struct TenantBinding<const MODE: u64>;
type Scope<const MODE: u64> = ApplicationQueryFieldScope<
    DocumentRetentionSchema<MODE>,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl<const MODE: u64> ApplicationQueryBinding<DocumentRetentionSchema<MODE>>
    for TenantBinding<MODE>
{
    type Input = ReadTenant;
    type InputBinding = ReadTenantBinding;
    type Query = TenantQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = TenantRowBinding;
    type ScopeBinding = Scope<MODE>;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.body-court.link-query.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<MODE>,
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
        DocumentRetentionSchema<MODE>,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl<const MODE: u64> ApplicationQueryIntent<DocumentRetentionSchema<MODE>> for ReadTenant {
    type Binding = TenantBinding<MODE>;
    fn parameters(&self) -> ApplicationQueryParameterSet<TenantQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope<MODE> {
        Scope::new(DocumentIdentityField::reference(), self.0)
    }
}
