//! Recorded-proposer admission does not replace the ordinary document grant.
use super::super::{grants::recorded_operation::RecordedOperationCapability, schema::*};
use super::bindings::GuardedBinding;
use worth_query_host::facade::{
    declaration::application_operation::*, declaration::application_schema::*, primary_graph,
};
pub struct ActorBinding;
impl ApplicationMutationBinding<CapabilitySchema> for ActorBinding {
    const REQUIRES_WORKFLOW_AUTHORITY: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::occurrence_only()
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
    type ScopeBinding =
        <GuardedBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "capability-workflow.ActorBinding.v1";
    const HANDLER_IDENTITY: &'static str = "capability-workflow.handler.ActorBinding";
    const IDEMPOTENCY_IDENTITY: &'static str = "capability-workflow.key.ActorBinding";
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
impl ApplicationCapabilityMutationBinding<CapabilitySchema> for ActorBinding {
    type Capability = RecordedOperationCapability;
}
impl ApplicationMutationAdmissionBinding<CapabilitySchema> for ActorBinding {
    type Admission = ApplicationCapabilityAdmission<RecordedOperationCapability>;
}
