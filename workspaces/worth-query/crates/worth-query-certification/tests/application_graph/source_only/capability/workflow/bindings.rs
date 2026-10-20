//! Distinct guarded/control contracts over existing retention and revocation request contexts.
use super::super::{grants::*, schema::*};

use worth_query_host::facade::{
    declaration::application_operation::*, declaration::application_schema::*, primary_graph,
};
pub struct GuardedBinding;
impl ApplicationMutationBinding<CapabilitySchema> for GuardedBinding {
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
    const IDENTITY: &'static str = "capability-workflow.GuardedBinding.v1";
    const HANDLER_IDENTITY: &'static str = "capability-workflow.handler.GuardedBinding";
    const IDEMPOTENCY_IDENTITY: &'static str = "capability-workflow.key.GuardedBinding";

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
impl ApplicationCapabilityMutationBinding<CapabilitySchema> for GuardedBinding {
    type Capability = RetentionCapability;
}
impl ApplicationMutationAdmissionBinding<CapabilitySchema> for GuardedBinding {
    type Admission = ApplicationCapabilityAdmission<RetentionCapability>;
}
pub struct ControlBinding;
impl ApplicationMutationBinding<CapabilitySchema> for ControlBinding {
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
    const IDENTITY: &'static str = "capability-workflow.ControlBinding.v1";
    const HANDLER_IDENTITY: &'static str = "capability-workflow.handler.ControlBinding";
    const IDEMPOTENCY_IDENTITY: &'static str = "capability-workflow.key.ControlBinding";
    // Closed propose/effect/done graph: eight native creates and fifteen links.
    // Reuse the finite small kernel profile; domain writer limits remain separate.

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
impl ApplicationCapabilityMutationBinding<CapabilitySchema> for ControlBinding {
    type Capability = RetentionCapability;
}
impl ApplicationMutationAdmissionBinding<CapabilitySchema> for ControlBinding {
    type Admission = ApplicationCapabilityAdmission<RetentionCapability>;
}

#[path = "advance_binding.rs"]
mod advance;
pub use advance::AdvanceBinding;

#[path = "start_binding.rs"]
mod start;
pub use start::StartBinding;
