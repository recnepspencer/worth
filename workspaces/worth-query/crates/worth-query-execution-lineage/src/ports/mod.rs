//! Ports connecting primary-graph lineage, demand, and projection to higher execution layers.

mod attempt;
mod lineage;
mod query;

pub use attempt::{
    AdmittedOperationPort, ApplicationAttemptPort, BasisLeasePort, CommitAuthorityAdmissionPort,
    CoupledMemberPort, CoupledPreparationView, CurrentProofPort, StableObservedSourceFactsPort,
    TypedAdmittedOperationPort, WorkflowActorEvidenceRuntimePort,
};
pub use lineage::{
    AppliedManagedSourcePort, EmptyManagedDerivedViewPort, LineageHomeRetirementDenial,
    ManagedDerivedViewCourtDoor, ManagedDerivedViewPort, ManagedViewPublicationRequestPort,
    PendingProducerPublicationPort, PreparingCommitPort, QueryGovernancePort,
    SourceReadClosurePort, UnpublishedIdempotencyPort, WorthQueryReadCleanupRuntime,
};
pub use query::{
    AppliedOutputsSelection, DiagnosticComparisonAdmissionPort, EmptyWorkflowRetentionPort,
    GraphIntegrationPort, ManagedIndexCausePort, SourceAuthorizationLocus, SourceQueryReaderPort,
    SourceReadSupportPort, WorkflowRetentionPort,
};

#[doc(hidden)]
pub trait ProducerEditionDeclarationPort {
    type Error;

    fn build_installed_producer_edition(
        &self,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    ) -> Result<
        crate::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition,
        Self::Error,
    >;
}

/// Crate-boundary port exposing owner-private `WorthQueryApplicationCommitReceipt::retained_preimage`
/// strictly to upper execution split crates. Never exported on `worth-query-execution::facade` or Host.
pub trait CommitReceiptPreimagePort {
    fn retained_preimage(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>;
}

/// Crate-boundary port exposing owner-private `WorthQueryApplicationCommitAuthorityBinding`
/// disclosure/input carriers strictly to upper execution split crates. Never exported on
/// `worth-query-execution::facade` or Host.
pub trait CommitAuthorityBindingInternalPort {
    fn retained_governed_input_carrier(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::application_attempt::WorthQueryRetainedGovernedInput>;

    fn original_inspection_disclosure(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure>;

    fn retain_original_inspection_disclosure(
        &self,
    ) -> Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure,
        >,
    >;
}

/// Crate-boundary port exposing owner-private `WorthQueryRetainedGovernedInput` payload
/// accessors strictly to upper execution split crates. Never exported on
/// `worth-query-execution::facade` or Host.
pub trait RetainedGovernedInputInternalPort: Sized {
    fn from_admission<Input>(
        input: &worth_query_declaration::facade::application_operation::ApplicationSharedInput<
            Input,
        >,
        governed_identity: Option<[u8; 32]>,
        original_scope: Option<
            &std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector,
            >,
        >,
        correction: Option<
            &std::sync::Arc<
                crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure,
            >,
        >,
        captured: Option<&std::sync::Arc<dyn std::any::Any + Send + Sync>>,
        material: Option<(
            &worth_query_declaration::facade::application_operation::ApplicationCanonicalInput,
            &'static str,
            [u8; 32],
        )>,
    ) -> Self
    where
        Input: Send + Sync + 'static;

    fn material_input(
        &self,
    ) -> Option<(
        &worth_query_declaration::facade::application_operation::ApplicationCanonicalInput,
        &'static str,
        [u8; 32],
    )>;

    fn retained_input<Binding>(
        &self,
    ) -> Option<
        &worth_query_declaration::facade::application_operation::ApplicationRetainedInput<Binding>,
    >
    where
        Binding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;

    fn governed_identity(&self) -> Option<[u8; 32]>;

    fn original_scope(
        &self,
    ) -> Option<
        &std::sync::Arc<crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector>,
    >;

    fn correction(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure>;

    fn axis_probe(governed_identity: [u8; 32]) -> Self;
}

/// Crate-boundary port exposing owner-private `WorthQueryApplicationEffectEntity`
/// internals strictly to upper execution split crates. Never exported on
/// `worth-query-execution::facade` or Host.
pub trait EffectEntityInternalPort<Schema, Entity>: Sized {
    fn from_parts(
        reference: worth_relational::facade::transactions::EntityReference,
        entity: String,
        created_effect: Option<usize>,
        program: std::sync::Arc<()>,
    ) -> Self;

    fn reference(&self) -> &worth_relational::facade::transactions::EntityReference;

    fn entity(&self) -> &str;

    fn created_effect(&self) -> Option<usize>;

    fn program(&self) -> &std::sync::Arc<()>;
}

/// Crate-boundary port exposing owner-private `WorthQueryApplicationCreatedRelation`
/// internals strictly to upper execution split crates. Never exported on
/// `worth-query-execution::facade` or Host.
pub trait CreatedRelationInternalPort<Schema, Relation, From, To>: Sized {
    fn from_parts(
        relation: worth_query_installation::facade::ApplicationRelationRef<
            Schema,
            Relation,
            From,
            To,
        >,
        created_effect: usize,
        program: std::sync::Arc<()>,
    ) -> Self;

    fn relation(
        &self,
    ) -> &worth_query_installation::facade::ApplicationRelationRef<Schema, Relation, From, To>;

    fn created_effect(&self) -> usize;

    fn program(&self) -> &std::sync::Arc<()>;
}

/// Crate-boundary port exposing owner-private `WorthQueryInvariantEntityIdentity`
/// fields strictly to upper execution split crates. Never exported on Host.
pub trait InvariantEntityIdentityInternalPort<Schema, Entity> {
    fn entity_id(&self) -> worth_relational::facade::identity::EntityId;
    fn kind(&self) -> worth_relational::facade::identity::KindId;
    fn entity(&self) -> &std::sync::Arc<str>;
    fn authority_identity(&self) -> u64;
    fn phase(
        &self,
    ) -> Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>;
}

/// Crate-boundary port exposing owner-private `WorthQueryInvariantGraphMember`
/// fields strictly to upper execution split crates. Never exported on Host.
pub trait InvariantGraphMemberInternalPort {
    fn key(&self) -> &worth_relational::facade::schema::NativeMemberKey;
    fn carrier_name(&self) -> &std::sync::Arc<str>;
}

/// Crate-boundary port pairing recovered World continuation with its owner-issued
/// cleanup receipt strictly when World itself publishes that continuation.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait ProductUnpublishedRecoveryInternalPort {
    fn continue_original_in_request_with_cleanup(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::domain_computation::primary_graph::home_original_progression::ContinuedOriginalOutcome,
        worth_runtime_world::facade::RuntimeWorldRecoveryDenial,
    >;
}
