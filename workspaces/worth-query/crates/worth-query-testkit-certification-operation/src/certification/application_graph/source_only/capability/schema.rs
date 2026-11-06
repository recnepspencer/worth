//! Authored retention meaning for this court; canonical P0 is untouched.
#[path = "schema/records.rs"]
pub(super) mod storage_model;
pub use storage_model::{CapabilityModel, ExternalMapping, Principal, Document, ExternalIdentity, PrincipalFacts, DocumentFacts, ExternalIdentityField, MappingStatusField, PrincipalIdentityField, DocumentIdentityField, DocumentRetentionField, MappingTarget};
pub use worth_query_testkit_document_retention::facade::document_retention_model::retention_entry::{
    DocumentRetentionWritten, DocumentRetentionWrittenBinding, SetRetentionDenial,
    SetRetentionDenialBinding, DOCUMENT_IDENTITY,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::schema::{
    SetRetentionInput, SetRetentionInputBinding,
};
pub use declaration::application_operation::*;
pub use declaration::application_schema::*;

pub use worth_query_host::facade::{declaration, *};

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct CapabilitySchema;
impl Fragment for CapabilitySchema {
    const NAME: &'static str = "CapabilitySchema";
    type Bindings = NoBindings;
    type Members = (CapabilityContribution,);
}
impl ApplicationSchema for CapabilitySchema {
    const OWNER: &'static str = "capability_retention_courtroom";
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
impl CapabilitySchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for CapabilitySchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct CapabilityContribution;
impl ApplicationSchemaContribution<CapabilitySchema> for CapabilityContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("capability-retention.application.v1");
    type Members = (CapabilityModel, Contracts);
}
impl CapabilityContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<CapabilitySchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<CapabilitySchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<CapabilitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<CapabilitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(super::resources::declare(super::grants::declare(schema))
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
            .application_mutation_binding::<SetRetentionBinding>()
            .application_mutation_binding::<super::workflow::bindings::GuardedBinding>()
            .application_mutation_binding::<super::workflow::actor_binding::ActorBinding>()
            .application_mutation_binding::<super::workflow::bindings::ControlBinding>()
            .application_mutation_binding::<super::workflow::bindings::StartBinding>()
            .application_mutation_binding::<super::workflow::bindings::AdvanceBinding>()
            .application_query_binding::<super::readback::ReadBinding>()
            .application_query(super::readback::definition())
            .application_resource(super::resources::definition()))
    }
}

pub struct DocumentPrincipalBinding;
impl DocumentPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        CapabilitySchema,
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

worth_query_operation!(pub SetRetention for CapabilitySchema, input SetRetentionInputBinding);

impl OperationExpectsFact<SetRetention> for DocumentIdentityField {}

pub struct SetRetentionBinding;
pub type Scope = ApplicationMutationFieldScope<DocumentIdentityField>;
impl ApplicationMutationBinding<CapabilitySchema> for SetRetentionBinding {
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Result = DocumentRetentionWritten;
    type ResultBinding = DocumentRetentionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = SetRetention;
    type Decision = primary_graph::WorthQueryInvariantMutationTarget<CapabilitySchema, Document>;
    type Denial = SetRetentionDenial;
    type DenialBinding = SetRetentionDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "capability-retention.set.v1";
    const HANDLER_IDENTITY: &'static str = "capability-retention.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "capability-retention.key.v1";

    fn scope_field() -> DocumentIdentityField {
        DocumentIdentityField::at("DocumentIdentityField")
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
impl ApplicationCapabilityMutationBinding<CapabilitySchema> for SetRetentionBinding {
    type Capability = super::grants::RetentionCapability;
}
impl ApplicationMutationAdmissionBinding<CapabilitySchema> for SetRetentionBinding {
    type Admission = ApplicationCapabilityAdmission<super::grants::RetentionCapability>;
}
