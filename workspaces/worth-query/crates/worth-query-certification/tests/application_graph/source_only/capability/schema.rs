//! Authored retention meaning for this court; canonical P0 is untouched.
pub use crate::document_retention_model::retention_entry::{
    DocumentRetentionWritten, DocumentRetentionWrittenBinding, SetRetentionDenial,
    SetRetentionDenialBinding, DOCUMENT_IDENTITY,
};
pub use crate::document_retention_model::schema::{SetRetentionInput, SetRetentionInputBinding};
use declaration::application_operation::*;
use declaration::application_schema::*;
use worth_query_host::facade::{declaration, *};

worth_query_application! {
    pub CapabilitySchema {
        owner: "capability_retention_courtroom", version: (1, 0),
        contributions: [CapabilityContribution],
    }
}
worth_query_application_contribution! {
    pub contribution CapabilityContribution in CapabilitySchema {
        identity: "capability-retention.application.v1",
        members: |schema| {
            super::grants::declare(schema).entity(ExternalMapping::reference()).entity(Principal::reference()).entity(Document::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference()).aspect(Document::reference(), DocumentFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference()).field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(Document::reference(), DocumentIdentityField::reference()).field(Document::reference(), DocumentRetentionField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .principal_binding(DocumentPrincipalBinding::reference())
                .operation(SetRetention::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(SetRetention::reference(), 8)
                .operation_read_field(SetRetention::reference(), DocumentIdentityField::reference())
                .operation_read_field(SetRetention::reference(), DocumentRetentionField::reference())
                .operation_write(SetRetention::reference(), DocumentRetentionField::reference())
                .operation_expected_fact(SetRetention::reference(), DocumentIdentityField::reference())
                .application_mutation_binding::<SetRetentionBinding>()
                .application_mutation_binding::<super::workflow::bindings::GuardedBinding>()
                .application_mutation_binding::<super::workflow::actor_binding::ActorBinding>()
                .application_mutation_binding::<super::workflow::bindings::ControlBinding>()
                .application_mutation_binding::<super::workflow::bindings::StartBinding>()
                .application_mutation_binding::<super::workflow::bindings::AdvanceBinding>().application_query_binding::<super::readback::ReadBinding>()
                .application_query(super::readback::definition())
        }
    }
}
worth_query_entity!(pub ExternalMapping for CapabilitySchema);
worth_query_entity!(pub Principal for CapabilitySchema);
worth_query_entity!(pub Document for CapabilitySchema);
worth_query_aspect!(pub ExternalIdentity for CapabilitySchema, ExternalMapping; identity = AspectIdentity(0x91764001), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for CapabilitySchema, Principal; identity = AspectIdentity(0x91764002), revision = AspectContractRevision(1));
worth_query_aspect!(pub DocumentFacts for CapabilitySchema, Document; identity = AspectIdentity(0x91764003), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for CapabilitySchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryExternalPrincipalIdentity => declaration::authentication::WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for CapabilitySchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryPrincipalMappingStatus => declaration::authentication::WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for CapabilitySchema, Principal, PrincipalFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentIdentityField for CapabilitySchema, Document, DocumentFacts: String => declaration::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentRetentionField for CapabilitySchema, Document, DocumentFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in CapabilitySchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub DocumentPrincipalBinding in CapabilitySchema, mapping ExternalMapping { identity: ExternalIdentityField, status: MappingStatusField, target: MappingTarget => Principal, principal_identity: PrincipalIdentityField });
worth_query_operation!(pub SetRetention for CapabilitySchema, input SetRetentionInputBinding);
worth_query_operation_reads!(SetRetention => [DocumentIdentityField, DocumentRetentionField]);
worth_query_operation_writes!(SetRetention => [DocumentRetentionField]);
impl OperationExpectsFact<SetRetention> for DocumentIdentityField {}

pub struct SetRetentionBinding;
type Scope = ApplicationMutationFieldScope<
    CapabilitySchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
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
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 2, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(1024),
        );
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
impl ApplicationCapabilityMutationBinding<CapabilitySchema> for SetRetentionBinding {
    type Capability = super::grants::RetentionCapability;
}
impl ApplicationMutationAdmissionBinding<CapabilitySchema> for SetRetentionBinding {
    type Admission = ApplicationCapabilityAdmission<super::grants::RetentionCapability>;
}
