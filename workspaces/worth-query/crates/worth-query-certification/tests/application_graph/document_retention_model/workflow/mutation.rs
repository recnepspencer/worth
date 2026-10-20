use worth_query_host::facade::declaration::{
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationFieldScope, ApplicationMutationIntent, NoApplicationMutationOutputs,
        NoApplicationMutationSource,
    },
    application_schema::{
        ApplicationFieldRef, ApplicationPrincipalBindingRef, ApplicationSchemaDeclarationBuilder,
        EqualityPredicate, NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
    },
};
use worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget;
use worth_query_host::facade::worth_query_structured_value_binding;

use super::super::schema::{
    Document, DocumentFacts, DocumentIdentityField, DocumentPrincipalBinding,
    DocumentRetentionSchema, ExternalMapping, Principal,
};
use super::declaration::{
    WorkflowDefinitionAuthoringCapability, WorkflowDefinitionAuthoringInput,
    WorkflowDefinitionAuthoringInputBinding, WorkflowDefinitionAuthoringOperation,
    WorkflowInstanceStartCapability, WorkflowInstanceStartInput, WorkflowInstanceStartInputBinding,
    WorkflowInstanceStartOperation,
};

#[derive(Clone, Debug)]
pub struct WorkflowDefinitionAuthoringIntent {
    pub input: WorkflowDefinitionAuthoringInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowDefinitionAuthoringAccepted;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowDefinitionAuthoringDenial {}

worth_query_structured_value_binding!(pub WorkflowDefinitionAuthoringAcceptedBinding for WorkflowDefinitionAuthoringAccepted {
    identity: "worth.query.certification.workflow-definition-authoring-accepted.v1"
});
worth_query_structured_value_binding!(pub WorkflowDefinitionAuthoringDenialBinding for WorkflowDefinitionAuthoringDenial {
    identity: "worth.query.certification.workflow-definition-authoring-denial.v1"
});

pub struct WorkflowDefinitionAuthoringBinding;

type WorkflowDefinitionAuthoringScope = ApplicationMutationFieldScope<
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<DocumentRetentionSchema> for WorkflowDefinitionAuthoringBinding {
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = WorkflowDefinitionAuthoringInput;
    type InputBinding = WorkflowDefinitionAuthoringInputBinding;
    type Result = WorkflowDefinitionAuthoringAccepted;
    type ResultBinding = WorkflowDefinitionAuthoringAcceptedBinding;
    type IdempotencyKey = u64;
    type Operation = WorkflowDefinitionAuthoringOperation;
    type Decision = WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>;
    type Denial = WorkflowDefinitionAuthoringDenial;
    type DenialBinding = WorkflowDefinitionAuthoringDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = WorkflowDefinitionAuthoringScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str =
        "worth.query.certification.workflow-definition-authoring-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.workflow-definition-authoring-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.workflow-definition-publication.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const WORKFLOW_CONTROL: bool = true;

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema,
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
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

impl ApplicationCapabilityMutationBinding<DocumentRetentionSchema>
    for WorkflowDefinitionAuthoringBinding
{
    type Capability = WorkflowDefinitionAuthoringCapability;
}

impl ApplicationMutationIntent<DocumentRetentionSchema> for WorkflowDefinitionAuthoringIntent {
    type Binding = WorkflowDefinitionAuthoringBinding;

    fn input(&self) -> &WorkflowDefinitionAuthoringInput {
        &self.input
    }

    fn scope_binding(&self) -> WorkflowDefinitionAuthoringScope {
        WorkflowDefinitionAuthoringScope::new(
            DocumentIdentityField::reference(),
            self.input.identity.clone(),
        )
    }
}

pub(super) fn install_binding(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .application_mutation_binding::<WorkflowDefinitionAuthoringBinding>()
        .application_mutation_binding::<WorkflowInstanceStartBinding>()
}

#[derive(Clone, Debug)]
pub struct WorkflowInstanceStartIntent {
    pub input: WorkflowInstanceStartInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowInstanceStartAccepted;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowInstanceStartDenial {}

worth_query_structured_value_binding!(pub WorkflowInstanceStartAcceptedBinding for WorkflowInstanceStartAccepted {
    identity: "worth.query.certification.workflow-instance-start-accepted.v1"
});
worth_query_structured_value_binding!(pub WorkflowInstanceStartDenialBinding for WorkflowInstanceStartDenial {
    identity: "worth.query.certification.workflow-instance-start-denial.v1"
});

pub struct WorkflowInstanceStartBinding;

impl ApplicationMutationBinding<DocumentRetentionSchema> for WorkflowInstanceStartBinding {
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = WorkflowInstanceStartInput;
    type InputBinding = WorkflowInstanceStartInputBinding;
    type Result = WorkflowInstanceStartAccepted;
    type ResultBinding = WorkflowInstanceStartAcceptedBinding;
    type IdempotencyKey = u64;
    type Operation = WorkflowInstanceStartOperation;
    type Decision = WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>;
    type Denial = WorkflowInstanceStartDenial;
    type DenialBinding = WorkflowInstanceStartDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = WorkflowDefinitionAuthoringScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.workflow-instance-start-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.workflow-instance-start-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.workflow-instance-start.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const WORKFLOW_CONTROL: bool = true;

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema,
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
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

impl ApplicationCapabilityMutationBinding<DocumentRetentionSchema>
    for WorkflowInstanceStartBinding
{
    type Capability = WorkflowInstanceStartCapability;
}

impl ApplicationMutationIntent<DocumentRetentionSchema> for WorkflowInstanceStartIntent {
    type Binding = WorkflowInstanceStartBinding;

    fn input(&self) -> &WorkflowInstanceStartInput {
        &self.input
    }

    fn scope_binding(&self) -> WorkflowDefinitionAuthoringScope {
        WorkflowDefinitionAuthoringScope::new(
            DocumentIdentityField::reference(),
            self.input.document_identity.clone(),
        )
    }
}

impl
    worth_query_decl::facade::application_operation::ApplicationMutationAdmissionBinding<
        DocumentRetentionSchema,
    > for WorkflowDefinitionAuthoringBinding
{
    type Admission =
        worth_query_decl::facade::application_operation::ApplicationCapabilityAdmission<
            WorkflowDefinitionAuthoringCapability,
        >;
}

impl
    worth_query_decl::facade::application_operation::ApplicationMutationAdmissionBinding<
        DocumentRetentionSchema,
    > for WorkflowInstanceStartBinding
{
    type Admission =
        worth_query_decl::facade::application_operation::ApplicationCapabilityAdmission<
            WorkflowInstanceStartCapability,
        >;
}
