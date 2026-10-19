//! An approved archival request emits exactly the installed external effect.
use crate::document_retention_model::{
    retention_entry::{SetRetentionBinding, SetRetentionDenial, SetRetentionDenialBinding},
    schema::*,
    workflow::WorkflowApprovalCapability,
};
use std::num::NonZeroU64;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*,
    worth_query_application_contribution, worth_query_effect, worth_query_operation,
    worth_query_operation_emits, worth_query_operation_reads, worth_query_structured_value_binding,
};
use worth_query_host::facade::{application_contribution::*, primary_graph::*};

pub struct ArchivePayload;
impl ApplicationStructuredValueBinding for ArchivePayload {
    type Value = SetRetentionInput;
    const IDENTITY_NAME: &'static str = "neutral.archive.payload.v1";
    fn validate(input: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        if input.identity.len() <= 256 && !input.identity.is_empty() {
            Ok(())
        } else {
            Err(ApplicationValueValidationDenial::rejected(
                Self::IDENTITY,
                "bounded document identity required",
            ))
        }
    }
}
impl ApplicationRetainedEffectBinding for ArchivePayload {
    fn retained_bytes(input: &Self::Value) -> u64 {
        input.identity.len() as u64 + 8
    }
}
impl ApplicationExternalEffectBinding for ArchivePayload {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new(crate::archive::PROTOCOL),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 512;
    fn external_effect_bytes(input: &Self::Value) -> Vec<u8> {
        format!("{}\n{}\n", input.identity, input.retention_days).into_bytes()
    }
}
worth_query_effect!(pub ArchiveEffect for DocumentRetentionSchema, payload ArchivePayload);
worth_query_operation!(pub ArchiveOperation for DocumentRetentionSchema, input SetRetentionInputBinding);
worth_query_operation_reads!(ArchiveOperation => [DocumentIdentityField, DocumentRetentionField]);
worth_query_operation_emits!(ArchiveOperation => [ArchiveEffect]);

pub fn inbound() -> ApplicationInboundOccurrenceBinding<ArchiveEffect> {
    let n = |value| NonZeroU64::new(value).unwrap();
    ApplicationInboundOccurrenceBinding::new(
        ApplicationInboundOccurrenceProtocol::new(
            BoundaryProtocolIdentity::new(crate::archive::PROTOCOL),
            BoundaryProtocolVersion::new(1),
        ),
        crate::archive::SOURCE,
        ApplicationInboundOccurrenceLimits {
            maximum_envelope_bytes: n(8192),
            maximum_verifier_work: n(300_000),
            maximum_payload_bytes: n(512),
            maximum_outstanding_dispatch_provenance: n(8),
            maximum_accepted_occurrences: n(8),
            maximum_accepted_bytes: n(65536),
            maximum_concurrent_publications: n(1),
            maximum_discovery_work: n(128),
            replay_window_milliseconds: n(60000),
            maximum_cleanup_work: n(128),
        },
    )
    .unwrap()
}
worth_query_application_contribution! {
    pub contribution ArchiveContribution in DocumentRetentionSchema {
        identity: "neutral.observable.archive.v1",
        members: |schema| { schema
            .effect(ArchiveEffect::reference())
            .operation(ArchiveOperation::reference().definition()
            .external_effect_with_inbound(ArchiveEffect::reference(), WorthQueryExternalEffectCorrelationFamily::new(crate::archive::FAMILY).unwrap(), inbound())
            .no_aftermath().finish())
            .operation_projection_work_budget(ArchiveOperation::reference(), 8)
            .operation_read_field(ArchiveOperation::reference(), DocumentIdentityField::reference())
            .operation_read_field(ArchiveOperation::reference(), DocumentRetentionField::reference())
            .operation_emit(ArchiveOperation::reference(), ArchiveEffect::reference())
            .application_mutation_binding::<ArchiveBinding>() }
    }
}
impl WorthQueryApplicationContribution<DocumentRetentionSchema> for ArchiveContribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, DocumentRetentionSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<ArchiveBinding, _>(ArchiveHandler)
    }
}
#[derive(Clone)]
pub struct ArchiveIntent {
    pub input: SetRetentionInput,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ArchiveRequested {
    pub retention_days: u64,
}
worth_query_structured_value_binding!(pub ArchiveRequestedBinding for ArchiveRequested { identity: "neutral.archive.requested.v1" });
pub struct ArchiveBinding;
type Scope =
    <SetRetentionBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::ScopeBinding;
impl ApplicationMutationBinding<DocumentRetentionSchema> for ArchiveBinding {
    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Result = ArchiveRequested;
    type ResultBinding = ArchiveRequestedBinding;
    type IdempotencyKey = u64;
    type Operation = ArchiveOperation;
    type Decision = WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>;
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
    const IDENTITY: &'static str = "neutral.archive.binding.v1";
    const HANDLER_IDENTITY: &'static str = "neutral.archive.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "neutral.archive.key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const REQUIRES_WORKFLOW_AUTHORITY: bool = true;
    const CANDIDATES:
        worth_query_decl::facade::application_operation::ApplicationCandidateContract =
        worth_query_decl::facade::application_operation::ApplicationCandidateContract::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 1),
            ApplicationCandidateResourceCeiling::representation_bytes(4096),
        );
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::requiring_approval::<WorkflowApprovalCapability>()
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
        SetRetentionBinding::scope_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        SetRetentionBinding::principal_binding()
    }
}
impl ApplicationMutationAdmissionBinding<DocumentRetentionSchema> for ArchiveBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationIntent<DocumentRetentionSchema> for ArchiveIntent {
    type Binding = ArchiveBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.input
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(
            DocumentIdentityField::reference(),
            self.input.identity.clone(),
        )
    }
}
pub struct ArchiveFeature;
impl ApplicationFeature<DocumentRetentionSchema> for ArchiveFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "neutral.archive.feature.v1";
}
pub struct ArchiveDelivery;
impl ApplicationActionDelivery<DocumentRetentionSchema, ArchiveBinding> for ArchiveDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "neutral.archive.completion.v1";
}
pub type ArchiveAction = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    ArchiveFeature,
    ArchiveBinding,
    ArchiveDelivery,
>;
#[path = "effect/handler.rs"]
mod handler;
use handler::ArchiveHandler;

/// Counts actual example handler entries; neither grants nor drives an operation.
pub fn handler_invocations() -> [usize; 2] {
    handler::invocation_counts()
}
