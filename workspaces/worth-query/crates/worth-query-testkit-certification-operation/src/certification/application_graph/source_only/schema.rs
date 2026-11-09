//! Authored retention meaning for this court; canonical P0 is untouched.
#[path = "schema/records.rs"]
mod storage_model;
pub use storage_model::*;
pub use worth_query_testkit_document_retention::facade::document_retention_model::retention_entry::{
    DocumentRetentionWritten, DocumentRetentionWrittenBinding, SetRetentionDenial,
    SetRetentionDenialBinding, DOCUMENT_IDENTITY,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::schema::{
    SetRetentionInput, SetRetentionInputBinding,
};
pub use declaration::application_schema::*;
pub use worth_query_decl::facade::worth_query_mutation_binding;
pub use worth_query_host::facade::{declaration, *};

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct SourceOnlyRetentionSchema;
impl Fragment for SourceOnlyRetentionSchema {
    const NAME: &'static str = "SourceOnlyRetentionSchema";
    type Bindings = NoBindings;
    type Members = (SourceOnlyRetentionContribution,);
}
impl ApplicationSchema for SourceOnlyRetentionSchema {
    const OWNER: &'static str = "source_only_retention_courtroom";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl SourceOnlyRetentionSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for SourceOnlyRetentionSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct SourceOnlyRetentionContribution;
impl ApplicationSchemaContribution<SourceOnlyRetentionSchema> for SourceOnlyRetentionContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("source-only-retention.application.v1");
    type Members = (RetentionModel, Contracts);
}
impl SourceOnlyRetentionContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<SourceOnlyRetentionSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<SourceOnlyRetentionSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SourceOnlyRetentionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SourceOnlyRetentionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .principal_binding(DocumentPrincipalBinding::reference())
            .operation(
                SetRetention::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(SetRetention::reference(), 8)
            .operation_read_field(
                SetRetention::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .operation_read_field(
                SetRetention::reference(),
                DocumentRetentionField::at("DocumentRetentionField"),
            )
            .operation_write(
                SetRetention::reference(),
                DocumentRetentionField::at("DocumentRetentionField"),
            )
            .operation_expected_fact(
                SetRetention::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .operation_expected_fact(
                SetRetention::reference(),
                DocumentRetentionField::at("DocumentRetentionField"),
            )
            .application_mutation_binding::<SetRetentionBinding>()
            .application_query_binding::<super::readback::ReadBinding>()
            .application_query(super::readback::definition()))
    }
}

pub struct DocumentPrincipalBinding;
impl DocumentPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        SourceOnlyRetentionSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "DocumentPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

worth_query_operation!(pub SetRetention for SourceOnlyRetentionSchema, input SetRetentionInputBinding);

impl OperationExpectsFact<SetRetention> for DocumentIdentityField {}
impl OperationExpectsFact<SetRetention> for DocumentRetentionField {}

pub fn document_identity(input: &SetRetentionInput) -> String {
    input.identity.clone()
}
worth_query_mutation_binding!(
    pub SetRetentionBinding for SetRetentionInput, schema SourceOnlyRetentionSchema,
    identity "source-only-retention.set.v1", input SetRetentionInputBinding,
    operation SetRetention, result DocumentRetentionWrittenBinding,
    idempotency u64, identity "source-only-retention.key.v1",
    decision primary_graph::WorthQueryInvariantMutationTarget<SourceOnlyRetentionSchema, Document>, denial SetRetentionDenialBinding,
    handler identity "source-only-retention.handler.v1",
    outputs declaration::application_operation::NoApplicationMutationOutputs,
    principal DocumentPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DocumentIdentityField,
    field DocumentIdentityField::at("DocumentIdentityField"), value document_identity,
);
