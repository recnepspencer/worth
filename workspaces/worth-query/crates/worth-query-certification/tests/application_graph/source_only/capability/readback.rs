//! Independent committed read truth, through the public query owner.
use super::schema::*;
pub use crate::document_retention_model::schema::{
    DocumentQueryParameters, DocumentQueryParametersBinding, DocumentRetentionRow,
    DocumentRetentionRowBinding,
};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    worth_query_application_query, worth_query_portable_type,
};
pub struct RetentionSlot;
pub struct IdentitySlot;
worth_query_portable_type!(RetentionSlot => "source-only-retention.result-slot.v1");
worth_query_portable_type!(IdentitySlot => "source-only-retention.identity-slot.v1");
worth_query_application_query!(pub RetentionQuery for CapabilitySchema, identity "source-only-retention.read.v1", parameters DocumentQueryParametersBinding, result DocumentRetentionRowBinding, scope Document => "Document", name "retention");
fn retention_field() -> ApplicationQueryResultFieldRef<
    RetentionQuery,
    RetentionSlot,
    CapabilitySchema,
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
    CapabilitySchema,
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
    CapabilitySchema,
    RetentionQuery,
    DocumentQueryParameters,
    DocumentRetentionRow,
    Document,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        CapabilitySchema,
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
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 5))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<CapabilitySchema, RetentionQuery> for DocumentRetentionRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, CapabilitySchema, RetentionQuery>,
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
    CapabilitySchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<CapabilitySchema> for ReadBinding {
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
        CapabilitySchema,
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
        CapabilitySchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl ApplicationQueryIntent<CapabilitySchema> for ReadRetention {
    type Binding = ReadBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<RetentionQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ReadScope {
        ReadScope::new(DocumentIdentityField::reference(), self.identity)
    }
}
/// An independent installation-minted native reader observes the actual grant field.
pub fn grant_status(host: &super::host::Host) -> u64 {
    host.application
        .runtime()
        .with_host_advancement(|phase| {
            let observed = host
                .projection
                .project(
                    &phase,
                    |reader| {
                        let grant = reader
                            .resolve_entity(
                                super::grants::GrantIdentity::reference(),
                                "source-child".to_owned(),
                            )
                            .unwrap();
                        reader
                            .field(&grant, super::grants::GrantStatus::reference())
                            .unwrap()
                    },
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            *observed.output()
        })
        .unwrap()
}
