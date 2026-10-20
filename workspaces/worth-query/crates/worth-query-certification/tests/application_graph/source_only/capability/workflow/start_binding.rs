//! Native instance-start command selects its separately installed capability.
use super::*;
use worth_query_decl::facade::application_operation::ApplicationCandidatePolicy;
pub struct StartBinding;
impl ApplicationMutationBinding<CapabilitySchema> for StartBinding {
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
    const IDENTITY: &'static str = "capability-workflow.StartBinding.v1";
    const HANDLER_IDENTITY: &'static str = "capability-workflow.handler.StartBinding";
    const IDEMPOTENCY_IDENTITY: &'static str = "capability-workflow.key.StartBinding";
    // Closed propose/effect/done graph: eight native creates and fifteen links.
    // Reuse the finite small kernel profile; domain writer limits remain separate.
    const CANDIDATES: ApplicationCandidatePolicy =
        <crate::document_retention_model::workflow::WorkflowInstanceStartBinding as ApplicationMutationBinding<
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
impl ApplicationCapabilityMutationBinding<CapabilitySchema> for StartBinding {
    type Capability = WorkflowInstanceStartCapability;
}
impl ApplicationMutationAdmissionBinding<CapabilitySchema> for StartBinding {
    type Admission = ApplicationCapabilityAdmission<WorkflowInstanceStartCapability>;
}
