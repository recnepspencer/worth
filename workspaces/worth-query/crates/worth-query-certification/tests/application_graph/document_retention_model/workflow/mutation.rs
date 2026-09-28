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
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            // A 10k control graph needs <30k creates, <68k links and <100k
            // writes while keeping the installed aggregate below 200k items.
            ApplicationCandidateCardinalityCeiling::fixed(30_000, 0, 68_000, 2, 100_000, 0),
            ApplicationCandidateResourceCeiling::bounded(128 * 1024 * 1024, 20_000_000),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        content_identity(&key.to_le_bytes())
    }

    fn input_identity(input: &WorkflowDefinitionAuthoringInput) -> [u8; 32] {
        let mut bytes = input.identity.as_bytes().to_vec();
        bytes.extend_from_slice(&input.retention_days.to_le_bytes());
        content_identity(&bytes)
    }

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
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(64, 0, 128, 2, 512, 0),
            ApplicationCandidateResourceCeiling::bounded(2 * 1024 * 1024, 1_048_576),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        content_identity(&key.to_le_bytes())
    }

    fn input_identity(input: &WorkflowInstanceStartInput) -> [u8; 32] {
        content_identity(input.document_identity.as_bytes())
    }

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

fn content_identity(bytes: &[u8]) -> [u8; 32] {
    let mut identity = [0_u8; 32];
    let mut accumulator = 0xcbf2_9ce4_8422_2325_u64;
    for (index, byte) in bytes.iter().enumerate() {
        accumulator = (accumulator ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        identity[index % identity.len()] ^= (accumulator >> ((index % 8) * 8)) as u8;
    }
    identity
}
