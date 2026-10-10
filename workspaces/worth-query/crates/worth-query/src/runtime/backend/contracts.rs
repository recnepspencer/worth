use super::{
    LiveViewDeclarationAdmissionBoundaryReceipt, LiveViewDeclarationAdmissionReceipt,
    SubscriptionActivationReceipt, WriteAuthorityExecutionReceipt,
};
use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::memory_workspace::{
    WorthQueryEntity, WorthQueryEntityIdentity, WorthQueryLivePatch, WorthQueryLiveViewHandle,
    WorthQueryMutationKind, WorthQueryMutationReceipt, WorthQuerySnapshotIdentity,
    WorthQueryWorkspaceError,
};
use crate::program::WorthQueryDerivedView;
use crate::schema_view::QuerySchemaView;
use crate::session_label::WorthQuerySessionLabel;
use crate::subscription::SubscriptionActivationInput;
use crate::view_shape_live::WorthQueryGroupedBaselineMember;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::BridgeAdmittedWritebackExecution;
use worth_runtime_bridge::facade::{BridgeMutationAuthorityBundle, RuntimeBridge};

use super::{WorthQueryBackendInspectionError, WorthQueryBackendMergeAuthority};
use super::{WorthQueryPrimaryGraphBackendHandle, WorthQueryUnpublishedPrimaryGraphRuntime};

use crate::runtime::{
    WorthQueryBackendAdmissibleMutation, WorthQueryEffectPolicy,
    WorthQueryExistingTruthAssertionDenial, WorthQueryExistingTruthBindingDenial,
    WorthQueryExistingTruthProbe, WorthQueryExistingTruthProbeDenial,
    WorthQueryExistingTruthProbeField, WorthQueryExistingTruthProbeRequest,
    WorthQueryExistingTruthTargetBinding, WorthQueryIntentDeclaration, WorthQueryIntentExecution,
    WorthQueryLiveArtifactTarget, WorthQueryPreviewBasisAdmission, WorthQueryRuntimeError,
    WorthQueryRuntimeEvidenceAuthority, WorthQueryRuntimeInspectionEvidence,
    WorthQueryRuntimeSupportProfile, WorthQueryVerifiedExistingTruthAssertion,
    WorthQueryWriteCommand, WorthQueryWriteReceipt,
};

pub trait WorthQueryRuntimeBackend:
    super::WorthQueryMergeSnapshotOwner + super::WorthQuerySettlementRecoveryBackend
{
    fn support_profile(&self) -> WorthQueryRuntimeSupportProfile;
    #[doc(hidden)]
    fn prepare_product_source(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_query_execution::facade::integration::WorthQueryProductRelationalInstallation,
        super::WorthQueryProductSourceDenial,
    > {
        Err(super::WorthQueryProductSourceDenial::Unsupported)
    }
    #[doc(hidden)]
    fn readmits_primary_graph_source(
        &self,
        _installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    ) -> bool {
        false
    }
    #[doc(hidden)]
    fn rebind_primary_graph_source(
        &mut self,
        _installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
        _source_adapter: Box<dyn super::WorthQueryRuntimeSourceAdapter>,
    ) -> Result<(), WorthQueryWorkspaceError> {
        Err(WorthQueryWorkspaceError::new(
            "primary source rebind unsupported",
        ))
    }
    /// Transfers an unpublished Relational runtime into execution-owned
    /// primary-graph installation.
    /// The default denial makes custom backends incapable of accidentally
    /// claiming support for a construction protocol they do not implement.
    #[doc(hidden)]
    fn surrender_unpublished_primary_graph_runtime(
        &mut self,
    ) -> Result<WorthQueryUnpublishedPrimaryGraphRuntime, WorthQueryWorkspaceError> {
        Err(WorthQueryWorkspaceError::new(
            "this runtime backend cannot surrender an unpublished primary graph runtime",
        ))
    }

    /// Attaches the opaque shared handle only after execution publishes the
    /// primary graph.
    #[doc(hidden)]
    fn attach_primary_graph_runtime(
        &mut self,
        _runtime: WorthQueryPrimaryGraphBackendHandle,
    ) -> Result<(), WorthQueryWorkspaceError> {
        Err(WorthQueryWorkspaceError::new(
            "this runtime backend cannot attach an execution-owned primary graph",
        ))
    }

    fn current_snapshot_identity(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQuerySnapshotIdentity {
        super::unavailable_snapshot_identity()
    }

    fn admit_live_view_declaration(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError>;

    fn declare_live_view(
        &mut self,
        name: String,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError>;

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError>;

    fn write(
        &mut self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError>;

    fn write_batch(
        &mut self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutations: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<Vec<WorthQueryMutationReceipt>, WorthQueryWorkspaceError>;

    fn admit_existing_truth_binding(
        &self,
        _binding: &WorthQueryExistingTruthTargetBinding,
    ) -> Result<(), WorthQueryExistingTruthBindingDenial> {
        Ok(())
    }

    fn verify_existing_truth_assertion(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &WorthQueryExistingTruthTargetBinding,
        _aspects: &[crate::runtime::WorthQueryAuthoredAspectMutation],
    ) -> Result<WorthQueryVerifiedExistingTruthAssertion, WorthQueryExistingTruthAssertionDenial>
    {
        Err(WorthQueryExistingTruthAssertionDenial::new(
            binding,
            crate::runtime::WorthQueryExistingTruthAssertionDenialKind::BackendVerificationUnsupported,
            None,
            None,
            None,
            "this runtime backend does not admit backend-verified existing-truth assertions yet",
        ))
    }

    fn probe_existing_truth(
        &self,
        request: &WorthQueryExistingTruthProbeRequest,
    ) -> Result<WorthQueryExistingTruthProbe, WorthQueryExistingTruthProbeDenial> {
        Err(WorthQueryExistingTruthProbeDenial::new(
            request.binding(),
            crate::runtime::WorthQueryExistingTruthProbeDenialKind::BackendProbeUnsupported,
            None,
            "this runtime backend does not admit backend-verified existing-truth probes yet",
        ))
    }

    fn execute_intent(
        &mut self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryRuntimeError>;

    fn admit_query_writeback_authority(&self) -> Result<(), WorthQueryWorkspaceError> {
        Err(WorthQueryWorkspaceError::new(
            "this runtime backend has no admitted bridge writeback authority",
        ))
    }

    fn execute_query_writeback(
        &mut self,
        _declaration: &crate::workflow::QueryWritebackDeclaration,
    ) -> Result<
        BridgeAdmittedWritebackExecution,
        (crate::effect_lifecycle::EffectExecutionDenialKind, String),
    > {
        Err((
            crate::effect_lifecycle::EffectExecutionDenialKind::MissingBridgeAuthority,
            "this runtime backend has no admitted bridge writeback executor".to_string(),
        ))
    }

    fn capture_query_merge_authority(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _target_branch: &crate::runtime::WorthQueryAdmittedBranchName,
        _source_branch: &crate::runtime::WorthQueryAdmittedBranchName,
    ) -> Result<WorthQueryBackendMergeAuthority, WorthQueryWorkspaceError> {
        Err(WorthQueryWorkspaceError::new(
            "this runtime backend has no admitted relational merge authority",
        ))
    }

    fn validate_query_merge_authority(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _authority: &WorthQueryBackendMergeAuthority,
    ) -> Result<(), WorthQueryWorkspaceError> {
        Err(WorthQueryWorkspaceError::new(
            "this runtime backend cannot validate relational merge authority",
        ))
    }

    fn execute_query_merge(
        &mut self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _authority: &WorthQueryBackendMergeAuthority,
        _declaration: &crate::workflow::LoweredMergeWorkflowDeclaration,
    ) -> Result<
        worth_relational::facade::transactions::MergeExecutionOutcome,
        crate::effect_lifecycle::RelationalEffectExecutionFailure,
    > {
        Err((
            crate::effect_lifecycle::EffectExecutionDenialKind::MissingRelationalAuthority,
            "this runtime backend has no relational merge executor".to_string(),
        )
            .into())
    }

    fn execute_query_causal_inspection(
        &self,
        _plan: &crate::runtime::CausalInspectionPlan,
    ) -> Result<crate::runtime::QueryCausalInspectionArtifact, WorthQueryBackendInspectionError>
    {
        Err(WorthQueryBackendInspectionError::unavailable(
            "this runtime backend has no causal inspection materializer",
        ))
    }

    fn live_entities_for_target(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryEntity>;

    fn live_entities_for_granular_scope(
        &self,
        _target: &WorthQueryLiveArtifactTarget,
        _scope: &crate::live::WorthQueryMaintenanceScope,
        _basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<Vec<WorthQueryEntity>, WorthQueryWorkspaceError> {
        Err(WorthQueryWorkspaceError::new(
            "this runtime backend has no exact granular live-source reader",
        ))
    }

    fn collection_entity(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _collection: &str,
        _identity: &WorthQueryEntityIdentity,
    ) -> WorthQueryBackendEntityLookup {
        WorthQueryBackendEntityLookup::Unsupported
    }

    fn drain_live_patches_for_target(
        &mut self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch>;

    fn affected_live_view_targets(
        &self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget>;

    fn install_live_subscription(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationReceipt, WorthQueryWorkspaceError>;

    fn admit_preview_basis(
        &self,
        label: &WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError>;

    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError>;

    fn admit_preview_write_command(
        &self,
        _command: &WorthQueryWriteCommand,
    ) -> Result<(), WorthQueryWorkspaceError> {
        Ok(())
    }

    fn declaration_initialization_metadata(
        &self,
        _view: &WorthQueryDerivedView,
    ) -> Result<crate::runtime::WorthQueryMutationMetadata, WorthQueryWorkspaceError> {
        Ok(crate::runtime::WorthQueryMutationMetadata::default())
    }

    fn grouped_baseline_members(
        &self,
        _request: &DeclarativeLiveQueryRequest,
    ) -> Result<Option<Vec<WorthQueryGroupedBaselineMember>>, WorthQueryWorkspaceError> {
        Ok(None)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorthQueryBackendEntityLookup {
    Found(WorthQueryEntity),
    Absent,
    Unsupported,
}

pub trait WorthQueryRuntimeSchemaAdapter {
    fn build_live_view_declaration_admission_receipt(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
    ) -> LiveViewDeclarationAdmissionReceipt {
        LiveViewDeclarationAdmissionReceipt::from_request(name, request)
    }

    fn build_live_view_declaration_boundary_receipt(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        admission_receipt: LiveViewDeclarationAdmissionReceipt,
    ) -> LiveViewDeclarationAdmissionBoundaryReceipt {
        LiveViewDeclarationAdmissionBoundaryReceipt::from_request(name, request, admission_receipt)
    }

    fn admit_live_view(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError>;
}

pub trait WorthQueryRuntimeSnapshotIdentityAdapter {
    fn current_snapshot_identity(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQuerySnapshotIdentity;
}

pub trait WorthQueryRuntimeExistingTruthVerificationAdapter {
    fn verify_existing_truth_assertion(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &WorthQueryExistingTruthTargetBinding,
        aspects: &[crate::runtime::WorthQueryAuthoredAspectMutation],
    ) -> Result<(), WorthQueryExistingTruthAssertionDenial>;

    fn probe_existing_truth(
        &self,
        request: &WorthQueryExistingTruthProbeRequest,
    ) -> Result<Vec<WorthQueryExistingTruthProbeField>, WorthQueryExistingTruthProbeDenial>;
}

#[path = "write_authority_adapter.rs"]
mod write_authority_adapter;
pub use write_authority_adapter::WorthQueryRuntimeWriteAuthorityAdapter;
