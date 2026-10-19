//! Distinct guarded/control contracts over existing retention and revocation request contexts.
use super::super::super::{grants::*, schema::*};
use worth_query_host::facade::{
    declaration::application_operation::*, declaration::application_schema::*, primary_graph,
};
pub struct AdvanceBinding;
impl ApplicationMutationBinding<CapabilitySchema> for AdvanceBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    const WORKFLOW_CONTROL: bool = true;
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
    type ScopeBinding = ApplicationMutationFieldScope<
        CapabilitySchema,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "capability-workflow.AdvanceBinding.v1";
    const HANDLER_IDENTITY: &'static str = "capability-workflow.handler.AdvanceBinding";
    const IDEMPOTENCY_IDENTITY: &'static str = "capability-workflow.key.AdvanceBinding";
    // Native progression/acceptance uses the existing finite workflow profile.
    const CANDIDATES: worth_query_decl::facade::application_operation::ApplicationCandidateContract =
        <crate::document_retention_model::workflow::WorkflowAdvanceBinding as ApplicationMutationBinding<
            crate::document_retention_model::schema::DocumentRetentionSchema,
        >>::CANDIDATES;

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
impl ApplicationCapabilityMutationBinding<CapabilitySchema> for AdvanceBinding {
    type Capability = WorkflowAdvanceCapability;
}
impl ApplicationMutationAdmissionBinding<CapabilitySchema> for AdvanceBinding {
    type Admission = ApplicationCapabilityAdmission<WorkflowAdvanceCapability>;
}
