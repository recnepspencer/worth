//! Independent committed read truth, through the public query owner.
use super::schema::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
pub use crate::document_retention_model::schema::{
    DocumentQueryParameters, DocumentQueryParametersBinding, DocumentRetentionRow,
    DocumentRetentionRowBinding,
};
use worth_query_host::facade::{
    application_entry::*,
    declaration::{application_query::*, application_schema::*},
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow, WorthQueryPrimaryGraphApplicationRuntime,
    },
    product::WorthQueryProductBranch,
    worth_query_application_query, worth_query_portable_type,
};
pub struct RetentionSlot;
pub struct IdentitySlot;
worth_query_portable_type!(RetentionSlot => "source-only-retention.result-slot.v1");
worth_query_portable_type!(IdentitySlot => "source-only-retention.identity-slot.v1");
worth_query_application_query!(pub RetentionQuery for SourceOnlyRetentionSchema, identity "source-only-retention.read.v1", parameters DocumentQueryParametersBinding, result DocumentRetentionRowBinding, scope Document => "Document", name "retention");
fn retention_field() -> ApplicationQueryResultFieldRef<
    RetentionQuery,
    RetentionSlot,
    SourceOnlyRetentionSchema,
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
fn identity_field() -> ApplicationQueryResultFieldRef<
    RetentionQuery,
    IdentitySlot,
    SourceOnlyRetentionSchema,
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
pub fn definition() -> ApplicationQueryDefinition<
    SourceOnlyRetentionSchema,
    RetentionQuery,
    DocumentQueryParameters,
    DocumentRetentionRow,
    Document,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        SourceOnlyRetentionSchema,
        RetentionQuery,
        Document,
        DocumentRetentionRow,
        DocumentRetentionRowBinding,
    >::new(Document::reference())
    .field(retention_field())
    .field(identity_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(RetentionQuery::reference())
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
impl WorthQueryApplicationProjection<SourceOnlyRetentionSchema, RetentionQuery>
    for DocumentRetentionRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, SourceOnlyRetentionSchema, RetentionQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(identity_field())?,
            retention_days: row.field(retention_field())?,
        })
    }
}
pub use crate::document_retention_model::retention_entry::DocumentRetentionRead as ReadRetention;
pub struct ReadBinding;
type ReadScope = ApplicationQueryFieldScope<
    SourceOnlyRetentionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<SourceOnlyRetentionSchema> for ReadBinding {
    type Input = crate::document_retention_model::retention_entry::DocumentRetentionRead;
    type InputBinding =
        crate::document_retention_model::retention_entry::DocumentRetentionReadBinding;
    type Query = RetentionQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = DocumentRetentionRowBinding;
    type ScopeBinding = ReadScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "source-only-retention.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        SourceOnlyRetentionSchema,
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
        SourceOnlyRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl ApplicationQueryIntent<SourceOnlyRetentionSchema> for ReadRetention {
    type Binding = ReadBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<RetentionQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ReadScope {
        ReadScope::new(DocumentIdentityField::reference(), self.identity)
    }
}
pub fn read_retention(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<SourceOnlyRetentionSchema>,
    branch: WorthQueryProductBranch,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .query(ReadRetention {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .retention_days
}
pub fn observe_head(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<SourceOnlyRetentionSchema>,
    branch: WorthQueryProductBranch,
) -> WorthQueryApplicationReadObservation {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .with_host_advancement(|phase| {
            runtime
                .request(&principal, &scope)
                .on_branch(branch)
                .retain_read(&phase)
                .unwrap()
        })
        .unwrap()
}
