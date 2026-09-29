use worth_query_host::facade::declaration::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
        ApplicationCandidateResourceCeiling, ApplicationCapabilityMutationBinding,
        ApplicationMutationBinding, ApplicationMutationFieldScope, ApplicationMutationIntent,
        NoApplicationMutationOutputs, NoApplicationMutationSource,
    },
    application_schema::{
        ApplicationFieldRef, ApplicationPrincipalBindingRef, ApplicationSchemaDeclarationBuilder,
        EqualityPredicate, NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
    },
};
use worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget;
use worth_query_host::facade::worth_query_structured_value_binding;

use super::super::super::schema::{
    Document, DocumentFacts, DocumentIdentityField, DocumentPrincipalBinding,
    DocumentRetentionSchema, ExternalMapping, Principal,
};
use super::declaration::WorkflowAdvanceInputBinding;
use super::{
    WorkflowAdvanceCapability, WorkflowAdvanceInput, WorkflowAdvanceOperation,
    WorkflowApprovalCapability,
};

#[derive(Clone, Debug)]
pub struct WorkflowAdvanceIntent {
    pub input: WorkflowAdvanceInput,
}

#[derive(Clone, Debug)]
pub struct WorkflowApprovalIntent {
    pub input: WorkflowAdvanceInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowAdvanceAccepted;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowAdvanceDenial {}

worth_query_structured_value_binding!(pub WorkflowAdvanceAcceptedBinding for WorkflowAdvanceAccepted {
    identity: "worth.query.certification.workflow-advance-accepted.v1"
});
worth_query_structured_value_binding!(pub WorkflowAdvanceDenialBinding for WorkflowAdvanceDenial {
    identity: "worth.query.certification.workflow-advance-denial.v1"
});

pub struct WorkflowAdvanceBinding;
pub struct WorkflowApprovalBinding;

type WorkflowAdvanceScope = ApplicationMutationFieldScope<
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<DocumentRetentionSchema> for WorkflowAdvanceBinding {
    type Input = WorkflowAdvanceInput;
    type InputBinding = WorkflowAdvanceInputBinding;
    type Result = WorkflowAdvanceAccepted;
    type ResultBinding = WorkflowAdvanceAcceptedBinding;
    type IdempotencyKey = u64;
    type Operation = WorkflowAdvanceOperation;
    type Decision = WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>;
    type Denial = WorkflowAdvanceDenial;
    type DenialBinding = WorkflowAdvanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = WorkflowAdvanceScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.workflow-advance-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.workflow-advance-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.workflow-advance.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const WORKFLOW_CONTROL: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(8, 0, 9, 2, 71, 0),
            ApplicationCandidateResourceCeiling::bounded(256 * 1024, 131_072),
        );

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

impl ApplicationCapabilityMutationBinding<DocumentRetentionSchema> for WorkflowAdvanceBinding {
    type Capability = WorkflowAdvanceCapability;
}

impl ApplicationMutationBinding<DocumentRetentionSchema> for WorkflowApprovalBinding {
    type Input = WorkflowAdvanceInput;
    type InputBinding = WorkflowAdvanceInputBinding;
    type Result = WorkflowAdvanceAccepted;
    type ResultBinding = WorkflowAdvanceAcceptedBinding;
    type IdempotencyKey = u64;
    type Operation = WorkflowAdvanceOperation;
    type Decision = WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>;
    type Denial = WorkflowAdvanceDenial;
    type DenialBinding = WorkflowAdvanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = WorkflowAdvanceScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.workflow-approval-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.workflow-approval-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.workflow-approval.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const WORKFLOW_CONTROL: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements = WorkflowAdvanceBinding::CANDIDATES;

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

impl ApplicationCapabilityMutationBinding<DocumentRetentionSchema> for WorkflowApprovalBinding {
    type Capability = WorkflowApprovalCapability;
}

impl ApplicationMutationIntent<DocumentRetentionSchema> for WorkflowAdvanceIntent {
    type Binding = WorkflowAdvanceBinding;

    fn input(&self) -> &WorkflowAdvanceInput {
        &self.input
    }

    fn scope_binding(&self) -> WorkflowAdvanceScope {
        WorkflowAdvanceScope::new(
            DocumentIdentityField::reference(),
            self.input.document_identity.clone(),
        )
    }
}

impl ApplicationMutationIntent<DocumentRetentionSchema> for WorkflowApprovalIntent {
    type Binding = WorkflowApprovalBinding;

    fn input(&self) -> &WorkflowAdvanceInput {
        &self.input
    }

    fn scope_binding(&self) -> WorkflowAdvanceScope {
        WorkflowAdvanceScope::new(
            DocumentIdentityField::reference(),
            self.input.document_identity.clone(),
        )
    }
}

pub(super) fn install_binding(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .application_mutation_binding::<WorkflowAdvanceBinding>()
        .application_mutation_binding::<WorkflowApprovalBinding>()
}
