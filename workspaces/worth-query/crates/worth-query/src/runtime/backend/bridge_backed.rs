use worth_runtime_bridge::facade::RuntimeBridge;

use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::memory_workspace::{
    WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle, WorthQueryMutationReceipt,
    WorthQuerySnapshotIdentity, WorthQueryWorkspaceError,
};
use crate::schema_view::QuerySchemaView;
use crate::session_label::WorthQuerySessionLabel;
use crate::subscription::SubscriptionActivationInput;

use crate::runtime::{
    WorthQueryBackendAdmissibleMutation, WorthQueryBackendMergeAuthority, WorthQueryEffectPolicy,
    WorthQueryExistingTruthAssertionDenial, WorthQueryExistingTruthBindingDenial,
    WorthQueryExistingTruthProbe, WorthQueryExistingTruthProbeDenial,
    WorthQueryExistingTruthTargetBinding, WorthQueryIntentDeclaration, WorthQueryIntentExecution,
    WorthQueryLiveArtifactTarget, WorthQueryPreviewBasisAdmission, WorthQueryRuntimeBackend,
    WorthQueryRuntimeError, WorthQueryRuntimeEvidenceAuthority,
    WorthQueryRuntimeInspectionEvidence, WorthQueryRuntimeSupportProfile,
    WorthQueryVerifiedExistingTruthAssertion, WorthQueryWriteReceipt,
};

use super::bootstrap::BridgeBackedRuntimeBootstrap;

mod relational_execution;
mod settlement_recovery;

pub struct WorthQueryBridgeBackedRuntimeBackend {
    relational_runtime: Option<super::relational_owner::WorthQueryBackendRelationalOwner>,
    runtime_bridge: RuntimeBridge,
    schema_adapter: Box<dyn super::WorthQueryRuntimeSchemaAdapter>,
    source_adapter: Box<dyn super::WorthQueryRuntimeSourceAdapter>,
    snapshot_identity: Option<Box<dyn super::WorthQueryRuntimeSnapshotIdentityAdapter>>,
    existing_truth_verification:
        Option<Box<dyn super::WorthQueryRuntimeExistingTruthVerificationAdapter>>,
    write_authority: Box<dyn super::WorthQueryRuntimeWriteAuthorityAdapter>,
    signal_sink: Box<dyn super::WorthQueryRuntimeSignalSinkAdapter>,
    subscription_activation: Box<dyn super::WorthQueryRuntimeSubscriptionActivationAdapter>,
    preview_basis: Box<dyn super::WorthQueryRuntimePreviewBasisAdapter>,
    inspector_evidence: Box<dyn super::WorthQueryRuntimeInspectorEvidenceAdapter>,
    declaration_initialization:
        Option<Box<dyn super::WorthQueryRuntimeDeclarationInitializationAdapter>>,
    intent_authority: Option<Box<dyn super::WorthQueryIntentAuthorityAdapter>>,
    support_profile: WorthQueryRuntimeSupportProfile,
}

impl WorthQueryBridgeBackedRuntimeBackend {
    #[cfg(test)]
    pub(crate) fn from_parts(
        parts: super::WorthQueryRuntimeBackendParts,
    ) -> Result<Self, WorthQueryRuntimeError> {
        Ok(Self::from_validated_bootstrap(
            parts.lower_bridge_backed_bootstrap()?,
        ))
    }

    pub(in crate::runtime) fn from_validated_bootstrap(
        bootstrap: BridgeBackedRuntimeBootstrap,
    ) -> Self {
        Self {
            relational_runtime: bootstrap.relational_runtime,
            runtime_bridge: bootstrap.runtime_bridge,
            schema_adapter: bootstrap.schema_adapter,
            source_adapter: bootstrap.source_adapter,
            snapshot_identity: bootstrap.snapshot_identity,
            existing_truth_verification: bootstrap.existing_truth_verification,
            write_authority: bootstrap.write_authority,
            signal_sink: bootstrap.signal_sink,
            subscription_activation: bootstrap.subscription_activation,
            preview_basis: bootstrap.preview_basis,
            inspector_evidence: bootstrap.inspector_evidence,
            declaration_initialization: bootstrap.declaration_initialization,
            intent_authority: bootstrap.intent_authority,
            support_profile: bootstrap.support_profile,
        }
    }
}

impl super::WorthQueryMergeSnapshotOwner for WorthQueryBridgeBackedRuntimeBackend {
    fn release_query_merge_snapshot(
        &mut self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        self.release_backend_merge_snapshot(snapshot)
    }
}

impl WorthQueryRuntimeBackend for WorthQueryBridgeBackedRuntimeBackend {
    fn prepare_product_source(
        &self,
    ) -> Result<
        worth_query_execution::facade::integration::WorthQueryProductRelationalInstallation,
        super::WorthQueryProductSourceDenial,
    > {
        self.relational_runtime
            .as_ref()
            .ok_or(super::WorthQueryProductSourceDenial::SourceNotInstalled)?
            .prepare_product_source()
    }
    fn support_profile(&self) -> WorthQueryRuntimeSupportProfile {
        self.support_profile.clone()
    }

    fn readmits_primary_graph_source(
        &self,
        installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    ) -> bool {
        self.source_adapter
            .primary_graph_invalidation_installation()
            .is_some_and(|source| source.is_same_current_runtime_as(installation))
    }

    fn rebind_primary_graph_source(
        &mut self,
        installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
        source_adapter: Box<dyn super::WorthQueryRuntimeSourceAdapter>,
    ) -> Result<(), WorthQueryWorkspaceError> {
        let replacement_matches = source_adapter
            .primary_graph_invalidation_installation()
            .is_some_and(|source| source.is_same_current_runtime_as(installation));
        if !replacement_matches {
            return Err(WorthQueryWorkspaceError::new(
                "the replacement primary graph source does not retain the successor installation",
            ));
        }
        self.source_adapter = source_adapter;
        Ok(())
    }

    fn surrender_unpublished_primary_graph_runtime(
        &mut self,
    ) -> Result<super::WorthQueryUnpublishedPrimaryGraphRuntime, WorthQueryWorkspaceError> {
        self.surrender_unpublished_graph_runtime()
    }

    fn attach_primary_graph_runtime(
        &mut self,
        runtime: super::WorthQueryPrimaryGraphBackendHandle,
    ) -> Result<(), WorthQueryWorkspaceError> {
        self.attach_published_graph_runtime(runtime)
    }

    fn current_snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        match self.snapshot_identity.as_ref() {
            Some(adapter) => adapter.current_snapshot_identity(),
            None => super::unavailable_snapshot_identity(),
        }
    }

    fn declare_live_view(
        &mut self,
        name: String,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        self.source_adapter
            .declare_live_view(name, request, schema_view)
    }

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError> {
        self.source_adapter.close_live_view(name)
    }

    fn admit_live_view_declaration(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        schema_view: &QuerySchemaView,
    ) -> Result<super::LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError> {
        self.schema_adapter
            .admit_live_view(name, request, schema_view)
    }

    fn write(
        &mut self,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        self.execute_backend_write(mutation)
    }

    fn write_batch(
        &mut self,
        mutations: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<Vec<WorthQueryMutationReceipt>, WorthQueryWorkspaceError> {
        self.execute_backend_write_batch(mutations)
    }

    fn admit_existing_truth_binding(
        &self,
        _binding: &WorthQueryExistingTruthTargetBinding,
    ) -> Result<(), WorthQueryExistingTruthBindingDenial> {
        Ok(())
    }

    fn verify_existing_truth_assertion(
        &self,
        binding: &WorthQueryExistingTruthTargetBinding,
        aspects: &[crate::runtime::WorthQueryAuthoredAspectMutation],
    ) -> Result<WorthQueryVerifiedExistingTruthAssertion, WorthQueryExistingTruthAssertionDenial>
    {
        let Some(adapter) = self.existing_truth_verification.as_ref() else {
            return Err(WorthQueryExistingTruthAssertionDenial::new(
                binding,
                crate::runtime::WorthQueryExistingTruthAssertionDenialKind::BackendVerificationUnsupported,
                None,
                None,
                None,
                "this runtime backend does not admit backend-verified existing-truth assertions yet",
            ));
        };
        adapter.verify_existing_truth_assertion(binding, aspects)?;
        WorthQueryVerifiedExistingTruthAssertion::from_snapshot_identity(
            binding,
            aspects,
            &self.current_snapshot_identity(),
        )
        .map_err(|error| {
            WorthQueryExistingTruthAssertionDenial::new(
                binding,
                crate::runtime::WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
                None,
                None,
                None,
                error.to_string(),
            )
        })
    }

    fn probe_existing_truth(
        &self,
        request: &crate::runtime::WorthQueryExistingTruthProbeRequest,
    ) -> Result<WorthQueryExistingTruthProbe, WorthQueryExistingTruthProbeDenial> {
        let Some(adapter) = self.existing_truth_verification.as_ref() else {
            return Err(WorthQueryExistingTruthProbeDenial::new(
                request.binding(),
                crate::runtime::WorthQueryExistingTruthProbeDenialKind::BackendProbeUnsupported,
                None,
                "this runtime backend does not admit backend-verified existing-truth probes yet",
            ));
        };
        WorthQueryExistingTruthProbe::backend_verified(
            request,
            adapter.probe_existing_truth(request)?,
        )
    }

    fn execute_intent(
        &mut self,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryRuntimeError> {
        self.execute_backend_intent(declaration)
    }

    fn admit_query_writeback_authority(&self) -> Result<(), WorthQueryWorkspaceError> {
        if self.runtime_bridge.writeback_authority().is_some() {
            Ok(())
        } else {
            Err(WorthQueryWorkspaceError::new(
                "bridge-backed runtime has no configured truth writeback authority",
            ))
        }
    }

    fn execute_query_writeback(
        &mut self,
        declaration: &crate::workflow::QueryWritebackDeclaration,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeAdmittedWritebackExecution,
        (crate::effect_lifecycle::EffectExecutionDenialKind, String),
    > {
        crate::effect_lifecycle::execute_lowered_writeback(&self.runtime_bridge, declaration)
    }

    fn capture_query_merge_authority(
        &self,
        target_branch: &crate::runtime::WorthQueryAdmittedBranchName,
        source_branch: &crate::runtime::WorthQueryAdmittedBranchName,
    ) -> Result<WorthQueryBackendMergeAuthority, WorthQueryWorkspaceError> {
        self.capture_backend_merge_authority(target_branch, source_branch)
    }

    fn validate_query_merge_authority(
        &self,
        authority: &WorthQueryBackendMergeAuthority,
    ) -> Result<(), WorthQueryWorkspaceError> {
        self.validate_backend_merge_authority(authority)
    }

    fn execute_query_merge(
        &mut self,
        authority: &WorthQueryBackendMergeAuthority,
        declaration: &crate::workflow::LoweredMergeWorkflowDeclaration,
    ) -> Result<
        worth_relational::facade::transactions::MergeExecutionOutcome,
        crate::effect_lifecycle::RelationalEffectExecutionFailure,
    > {
        self.execute_backend_merge(authority, declaration)
    }

    fn execute_query_causal_inspection(
        &self,
        plan: &crate::runtime::CausalInspectionPlan,
    ) -> Result<
        crate::runtime::QueryCausalInspectionArtifact,
        crate::runtime::WorthQueryBackendInspectionError,
    > {
        plan.materialize_with_bridge(&self.runtime_bridge)
            .map_err(Into::into)
    }

    fn live_entities_for_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryEntity> {
        self.source_adapter.live_entities_for_target(target)
    }

    fn live_entities_for_granular_scope(
        &self,
        target: &WorthQueryLiveArtifactTarget,
        scope: &crate::live::WorthQueryMaintenanceScope,
        basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<Vec<WorthQueryEntity>, WorthQueryWorkspaceError> {
        self.source_adapter
            .live_entities_for_granular_scope(target, scope, basis)
    }

    fn drain_live_patches_for_target(
        &mut self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        self.source_adapter.drain_live_patches_for_target(target)
    }

    fn affected_live_view_targets(
        &self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        self.source_adapter.affected_live_view_targets(receipt)
    }

    fn install_live_subscription(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<super::SubscriptionActivationReceipt, WorthQueryWorkspaceError> {
        let activation_receipt = self
            .subscription_activation
            .admit_activation(view_name, activation)?;
        if let Some(message) = activation_receipt.drift_from_activation(view_name, activation) {
            return Err(WorthQueryWorkspaceError::new(message));
        }
        Ok(activation_receipt.activation_receipt().clone())
    }

    fn admit_preview_basis(
        &self,
        label: &WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError> {
        self.preview_basis
            .admit_preview_basis(label, effect_policy, authority)
    }

    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError> {
        self.inspector_evidence
            .inspect_write_receipt(receipt, authority)
    }

    fn declaration_initialization_metadata(
        &self,
        view: &crate::program::WorthQueryDerivedView,
    ) -> Result<crate::runtime::WorthQueryMutationMetadata, WorthQueryWorkspaceError> {
        match self.declaration_initialization.as_ref() {
            Some(adapter) => adapter.declaration_initialization_metadata(view),
            None => Ok(crate::runtime::WorthQueryMutationMetadata::default()),
        }
    }
}
