//! Crate-boundary ports for commit receipts, recovered receipts, and sealed
//! commit-authority minting. Never exported on `worth-query-execution::facade`
//! or Host.

use std::any::Any;
use std::sync::Arc;

use worth_proof::QueryReadmitted;

use crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch;
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_attempt::compare_and_commit::cleanup_required::WorthQueryApplicationCleanupRequired;
use crate::domain_computation::primary_graph::application_attempt::{
    OutputFieldCaptureFailure, WorthQueryApplicationCommitAuthorityBinding,
    WorthQueryApplicationCommitOutcomeIdentity, WorthQueryApplicationCommitPublicationSource,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationOutputCorrespondence, WorthQueryExternalDispatchPreparationDenial,
    WorthQueryRetainedGovernedInput,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::provider::idempotency::snapshot_resolution::original_receipt::sealing::{
    MemoryOriginalReceiptInputs, OriginalReceiptInputs,
};
use crate::domain_computation::primary_graph::provider::{
    FactlessCurrentness, WorthQueryCommittedDispatchOutboxBinding,
    WorthQueryPrimaryGraphCommitEvidence, WorthQueryProviderIdempotencyResolutionDenial,
    WorthQueryRecoveredApplicationCommitReceipt,
};
use crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure;

pub trait CommitReceiptInternalPort: Sized {
    fn performed_output_fields(&self) -> Option<&Result<FactPayload, OutputFieldCaptureFailure>>;
    fn performed_input_facts(&self) -> Option<&FactPayload>;
    fn currentness_without_facts_at(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Option<FactlessCurrentness>;
    fn retain_output_correspondence(&self) -> Arc<WorthQueryApplicationOutputCorrespondence>;
    fn exact_output_settlement(&self) -> Option<&Arc<RecordedSettlementIdentity>>;
    fn publication_source(&self) -> WorthQueryApplicationCommitPublicationSource;
    fn committed_dispatch_outbox(&self) -> Option<&WorthQueryCommittedDispatchOutboxBinding>;
    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity;
    fn installed_aftermath(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryInstalledAftermathContract>;
    fn with_external_dispatch(self, dispatch: WorthQueryExternalEffectDispatch) -> Self;
    fn with_external_dispatch_preparation_denial(
        self,
        denial: WorthQueryExternalDispatchPreparationDenial,
    ) -> Self;
    fn settle_for_ready_cache(&mut self);
}

impl CommitReceiptInternalPort for WorthQueryApplicationCommitReceipt {
    fn performed_output_fields(&self) -> Option<&Result<FactPayload, OutputFieldCaptureFailure>> {
        Self::performed_output_fields(self)
    }

    fn performed_input_facts(&self) -> Option<&FactPayload> {
        Self::performed_input_facts(self)
    }

    fn currentness_without_facts_at(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Option<FactlessCurrentness> {
        Self::currentness_without_facts_at(self, observation)
    }

    fn retain_output_correspondence(&self) -> Arc<WorthQueryApplicationOutputCorrespondence> {
        Self::retain_output_correspondence(self)
    }

    fn exact_output_settlement(&self) -> Option<&Arc<RecordedSettlementIdentity>> {
        Self::exact_output_settlement(self)
    }

    fn publication_source(&self) -> WorthQueryApplicationCommitPublicationSource {
        Self::publication_source(self)
    }

    fn committed_dispatch_outbox(&self) -> Option<&WorthQueryCommittedDispatchOutboxBinding> {
        Self::committed_dispatch_outbox(self)
    }

    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        Self::runtime_authority(self)
    }

    fn installed_aftermath(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryInstalledAftermathContract> {
        Self::installed_aftermath(self)
    }

    fn with_external_dispatch(self, dispatch: WorthQueryExternalEffectDispatch) -> Self {
        Self::with_external_dispatch(self, dispatch)
    }

    fn with_external_dispatch_preparation_denial(
        self,
        denial: WorthQueryExternalDispatchPreparationDenial,
    ) -> Self {
        Self::with_external_dispatch_preparation_denial(self, denial)
    }

    fn settle_for_ready_cache(&mut self) {
        Self::settle_for_ready_cache(self);
    }
}

pub trait RecoveredCommitReceiptInternalPort: Sized {
    fn retained_backing_bytes() -> usize;
    fn from_readmitted(
        original: QueryReadmitted<OriginalReceiptInputs>,
    ) -> Result<Self, WorthQueryProviderIdempotencyResolutionDenial>;
    fn from_memory(
        original: QueryReadmitted<MemoryOriginalReceiptInputs>,
    ) -> Result<Self, WorthQueryProviderIdempotencyResolutionDenial>;
}

impl RecoveredCommitReceiptInternalPort for WorthQueryRecoveredApplicationCommitReceipt {
    fn retained_backing_bytes() -> usize {
        Self::retained_backing_bytes()
    }

    fn from_readmitted(
        original: QueryReadmitted<OriginalReceiptInputs>,
    ) -> Result<Self, WorthQueryProviderIdempotencyResolutionDenial> {
        Self::from_readmitted(original)
    }

    fn from_memory(
        original: QueryReadmitted<MemoryOriginalReceiptInputs>,
    ) -> Result<Self, WorthQueryProviderIdempotencyResolutionDenial> {
        Self::from_memory(original)
    }
}

pub struct PrimaryGraphCommitEvidenceParts {
    pub workflow_origin: Option<
        crate::domain_computation::primary_graph::workflow::operation_output::WorkflowOperationOrigin,
    >,
    pub provider_session_binding:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    pub idempotency: WorthQueryApplicationIdempotencyBinding,
    pub commit: worth_relational::facade::history::RelationalCommitReceipt,
    pub mutation_work:
        crate::domain_computation::primary_graph::provider::WorthQueryPrimaryMutationWorkEvidence,
    pub retained_preimage:
        Option<crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>,
    pub original_authority: Arc<WorthQueryApplicationCommitAuthorityBinding>,
    pub committed_dispatch_outbox:
        crate::domain_computation::primary_graph::provider::session_commit::WorthQueryCommittedDispatchOutboxResolution,
    pub committed_changes:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommittedChanges,
    pub output_correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub operation_scope: WorthQueryOperationScopeBinding,
    pub observed_source_facts:
        crate::domain_computation::primary_graph::provider::RebasedSourceFacts,
    pub performed_input_facts: Option<FactPayload>,
    pub performed_output_fields: Option<Result<FactPayload, OutputFieldCaptureFailure>>,
}

pub trait PrimaryGraphCommitEvidenceInternalPort: Sized {
    fn from_parts(parts: PrimaryGraphCommitEvidenceParts) -> Self;
    fn performed_output_fields(&self) -> Option<&Result<FactPayload, OutputFieldCaptureFailure>>;
    fn performed_input_facts(&self) -> Option<&FactPayload>;
}

impl PrimaryGraphCommitEvidenceInternalPort for WorthQueryPrimaryGraphCommitEvidence {
    fn from_parts(parts: PrimaryGraphCommitEvidenceParts) -> Self {
        Self {
            workflow_origin: parts.workflow_origin,
            provider_session_binding: parts.provider_session_binding,
            idempotency: parts.idempotency,
            commit: parts.commit,
            mutation_work: parts.mutation_work,
            retained_preimage: parts.retained_preimage,
            original_authority: parts.original_authority,
            committed_dispatch_outbox: parts.committed_dispatch_outbox,
            committed_changes: parts.committed_changes,
            output_correspondence: parts.output_correspondence,
            operation_scope: parts.operation_scope,
            observed_source_facts: parts.observed_source_facts,
            performed_input_facts: parts.performed_input_facts,
            performed_output_fields: parts.performed_output_fields,
        }
    }

    fn performed_output_fields(&self) -> Option<&Result<FactPayload, OutputFieldCaptureFailure>> {
        Self::performed_output_fields(self)
    }

    fn performed_input_facts(&self) -> Option<&FactPayload> {
        Self::performed_input_facts(self)
    }
}

pub trait CommitAuthorityMintInternalPort: Sized {
    #[allow(clippy::too_many_arguments)]
    fn from_parts(
        original_mutation_binding: Option<std::any::TypeId>,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        installed_operation: [u8; 32],
        operation_definition_identity: [u8; 32],
        installed_aftermath: Option<
            worth_query_installation::facade::WorthQueryInstalledAftermathContract,
        >,
        principal_scope: WorthQueryOperationScopeBinding,
        idempotency_binding: WorthQueryApplicationIdempotencyBinding,
        retained_governed_input: Option<WorthQueryRetainedGovernedInput>,
        original_program: Option<Arc<dyn Any + Send + Sync>>,
        original_inspection_disclosure: Option<Arc<OriginalSourceInspectionDisclosure>>,
    ) -> Self;
}

impl CommitAuthorityMintInternalPort for WorthQueryApplicationCommitAuthorityBinding {
    fn from_parts(
        original_mutation_binding: Option<std::any::TypeId>,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        installed_operation: [u8; 32],
        operation_definition_identity: [u8; 32],
        installed_aftermath: Option<
            worth_query_installation::facade::WorthQueryInstalledAftermathContract,
        >,
        principal_scope: WorthQueryOperationScopeBinding,
        idempotency_binding: WorthQueryApplicationIdempotencyBinding,
        retained_governed_input: Option<WorthQueryRetainedGovernedInput>,
        original_program: Option<Arc<dyn Any + Send + Sync>>,
        original_inspection_disclosure: Option<Arc<OriginalSourceInspectionDisclosure>>,
    ) -> Self {
        Self::from_parts(
            original_mutation_binding,
            runtime_authority,
            installed_operation,
            operation_definition_identity,
            installed_aftermath,
            principal_scope,
            idempotency_binding,
            retained_governed_input,
            original_program,
            original_inspection_disclosure,
        )
    }
}

pub trait CommitOutcomeIdentityInternalPort: Sized {
    fn mint() -> Option<Self>;
}

impl CommitOutcomeIdentityInternalPort for WorthQueryApplicationCommitOutcomeIdentity {
    fn mint() -> Option<Self> {
        Self::mint()
    }
}

pub trait IdempotencyBindingInternalPort: Sized {
    fn from_identities(key_identity: [u8; 32], intent_identity: [u8; 32]) -> Self;
}

impl IdempotencyBindingInternalPort for WorthQueryApplicationIdempotencyBinding {
    fn from_identities(key_identity: [u8; 32], intent_identity: [u8; 32]) -> Self {
        Self::new(key_identity, intent_identity)
    }
}

pub trait OperationScopeBindingInternalPort: Sized {
    fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        operation_authority_identity: &str,
        principal: worth_relational::facade::identity::EntityId,
        scope: worth_relational::facade::identity::EntityId,
    ) -> Self;
}

impl OperationScopeBindingInternalPort for WorthQueryOperationScopeBinding {
    fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        operation_authority_identity: &str,
        principal: worth_relational::facade::identity::EntityId,
        scope: worth_relational::facade::identity::EntityId,
    ) -> Self {
        Self::mint(
            runtime_authority,
            binding_identity,
            operation_authority_identity,
            principal,
            scope,
        )
    }
}

pub trait CleanupRequiredInternalPort: Sized {
    fn from_parts(
        outcome: crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
        cleanup: crate::domain_computation::primary_graph::WorthQueryApplicationManagedCleanup,
    ) -> Box<Self>;

    fn retain_for_home(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        history: Arc<
            crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
        >,
    );
}

impl CleanupRequiredInternalPort for WorthQueryApplicationCleanupRequired {
    fn from_parts(
        outcome: crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
        cleanup: crate::domain_computation::primary_graph::WorthQueryApplicationManagedCleanup,
    ) -> Box<Self> {
        Self::from_parts(outcome, cleanup)
    }

    fn retain_for_home(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        history: Arc<
            crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
        >,
    ) {
        Self::retain_for_home(self, request, history)
    }
}
