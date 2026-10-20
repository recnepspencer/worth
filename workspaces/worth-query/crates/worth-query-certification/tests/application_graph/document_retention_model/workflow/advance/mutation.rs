use worth_query_decl::facade::application_operation::ApplicationCandidatePolicy;
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
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

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
    const CANDIDATES: ApplicationCandidatePolicy = ApplicationCandidatePolicy::finite({
        // Retain the six-source-fact allowance and the related-assessment
        // applicability's two endpoint kinds plus one adjacency revision.
        const DEPENDENCIES: usize = 6 + 3;
        // Current transition has seven structural fields and an advancer
        // triplet; Ready writes one instance field. Evidence has 19 fields.
        // Each dependency has four identity/tag fields and at most four
        // revision/locator fields (field or adjacency shape).
        const WRITES: usize = 7 + 3 + 1 + 19 + DEPENDENCIES * 8;
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(
                2 + DEPENDENCIES,
                0,
                3 + DEPENDENCIES,
                2,
                WRITES,
                0,
            ),
            ApplicationCandidateResourceCeiling::representation_bytes(256 * 1024),
        )
    });

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
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

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
    const CANDIDATES: ApplicationCandidatePolicy = WorkflowAdvanceBinding::CANDIDATES;

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

impl
    worth_query_decl::facade::application_operation::ApplicationMutationAdmissionBinding<
        DocumentRetentionSchema,
    > for WorkflowAdvanceBinding
{
    type Admission =
        worth_query_decl::facade::application_operation::ApplicationCapabilityAdmission<
            WorkflowAdvanceCapability,
        >;
}

impl
    worth_query_decl::facade::application_operation::ApplicationMutationAdmissionBinding<
        DocumentRetentionSchema,
    > for WorkflowApprovalBinding
{
    type Admission =
        worth_query_decl::facade::application_operation::ApplicationCapabilityAdmission<
            WorkflowApprovalCapability,
        >;
}
