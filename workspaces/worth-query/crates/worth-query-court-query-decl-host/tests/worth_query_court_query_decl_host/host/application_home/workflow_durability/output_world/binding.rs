//! Native output and workflow-control bindings use the declared schema and ordinary admission.
use super::*;
use worth_query_decl::facade::{
    worth_query_operation, worth_query_operation_reads, worth_query_structured_value_binding,
};
worth_query_operation!(pub OutputOperation for Schema, input SetRetentionInputBinding);
worth_query_operation!(pub ControlOperation for Schema, input SetRetentionInputBinding);
worth_query_operation_reads!(OutputOperation => [Document]);
worth_query_operation_reads!(ControlOperation => [Document]);
worth_query_structured_value_binding!(pub ResultBinding for () { identity: "durable-workflow.preserve-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for WorthQueryOperationBodyDenial { identity: "durable-workflow.preserve-denial.v1" });
pub struct Outputs;
pub struct DocumentOutput;
impl WorthQueryApplicationOutputRole for DocumentOutput {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Document;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "document";
}
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<DocumentOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
pub struct Binding;
impl ApplicationMutationBinding<Schema> for Binding {
    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Result = ();
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = OutputOperation;
    type Decision = WorthQueryOperationBodyDecision<Schema, Document>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = DenialBinding;
    type Output = Outputs;
    type ScopeBinding = ApplicationMutationFieldScope<DocumentIdentityField>;
    type PrincipalBinding = Authentication;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "durable-workflow.preserve-binding.v1";
    const HANDLER_IDENTITY: &'static str = "durable-workflow.preserve-body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "durable-workflow.preserve-command.v1";
    const REQUIRES_WORKFLOW_AUTHORITY: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::occurrence_only()
    }
    fn scope_field() -> DocumentIdentityField {
        DocumentIdentityField::at("DocumentIdentityField")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Authentication,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        authentication()
    }
}
impl ApplicationMutationAdmissionBinding<Schema> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
#[derive(Clone)]
pub struct Intent(pub SetRetentionInput);
impl ApplicationMutationIntent<Schema> for Intent {
    type Binding = Binding;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<DocumentIdentityField> {
        ApplicationMutationFieldScope::new(
            DocumentIdentityField::at("DocumentIdentityField"),
            self.0.identity.clone(),
        )
    }
}

pub struct ControlBinding<C = ControlCapability>(std::marker::PhantomData<C>);
impl<C: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static>
    ApplicationMutationBinding<Schema> for ControlBinding<C>
{
    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Result = ();
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = ControlOperation;
    type Decision = WorthQueryOperationBodyDecision<Schema, Document>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<DocumentIdentityField>;
    type PrincipalBinding = Authentication;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = C::IDENTIFIER;
    const HANDLER_IDENTITY: &'static str = "durable-workflow.native-control.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "durable-workflow.control-key.v1";
    const WORKFLOW_CONTROL: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> DocumentIdentityField {
        DocumentIdentityField::at("DocumentIdentityField")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Authentication,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        authentication()
    }
}
impl<C: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static>
    ApplicationMutationAdmissionBinding<Schema> for ControlBinding<C>
{
    type Admission = ApplicationCapabilityAdmission<C>;
}
impl<C: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static>
    ApplicationCapabilityMutationBinding<Schema> for ControlBinding<C>
{
    type Capability = C;
}
#[derive(Clone)]
pub struct Control(pub SetRetentionInput);
impl ApplicationMutationIntent<Schema> for Control {
    type Binding = ControlBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<DocumentIdentityField> {
        ApplicationMutationFieldScope::new(
            DocumentIdentityField::at("DocumentIdentityField"),
            self.0.identity.clone(),
        )
    }
}

#[derive(Clone)]
pub struct Start(pub SetRetentionInput);
impl ApplicationMutationIntent<Schema> for Start {
    type Binding = ControlBinding<StartCapability>;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<DocumentIdentityField> {
        ApplicationMutationFieldScope::new(
            DocumentIdentityField::at("DocumentIdentityField"),
            self.0.identity.clone(),
        )
    }
}
#[derive(Clone)]
pub struct Advance(pub SetRetentionInput);
impl ApplicationMutationIntent<Schema> for Advance {
    type Binding = ControlBinding<AdvanceCapability>;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<DocumentIdentityField> {
        ApplicationMutationFieldScope::new(
            DocumentIdentityField::at("DocumentIdentityField"),
            self.0.identity.clone(),
        )
    }
}
