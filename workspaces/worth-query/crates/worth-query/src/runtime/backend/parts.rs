use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::RuntimeBridge;

use crate::runtime::WorthQueryRuntimeError;

use super::{
    bootstrap::BridgeBackedRuntimeBootstrap, WorthQueryIntentAuthorityAdapter,
    WorthQueryRuntimeDeclarationInitializationAdapter,
    WorthQueryRuntimeExistingTruthVerificationAdapter, WorthQueryRuntimeInspectorEvidenceAdapter,
    WorthQueryRuntimePreviewBasisAdapter, WorthQueryRuntimeSchemaAdapter,
    WorthQueryRuntimeSignalSinkAdapter, WorthQueryRuntimeSnapshotIdentityAdapter,
    WorthQueryRuntimeSourceAdapter, WorthQueryRuntimeSubscriptionActivationAdapter,
    WorthQueryRuntimeWriteAuthorityAdapter,
};
use crate::runtime::WorthQueryRuntimeSupportProfile;

#[derive(Default)]
pub struct WorthQueryRuntimeBackendParts {
    pub(super) relational_runtime:
        Option<super::relational_owner::WorthQueryBackendRelationalOwner>,
    pub(super) runtime_bridge: Option<RuntimeBridge>,
    pub(super) schema_adapter: Option<Box<dyn WorthQueryRuntimeSchemaAdapter>>,
    pub(super) source_adapter: Option<Box<dyn WorthQueryRuntimeSourceAdapter>>,
    pub(super) snapshot_identity: Option<Box<dyn WorthQueryRuntimeSnapshotIdentityAdapter>>,
    pub(super) existing_truth_verification:
        Option<Box<dyn WorthQueryRuntimeExistingTruthVerificationAdapter>>,
    pub(super) write_authority: Option<Box<dyn WorthQueryRuntimeWriteAuthorityAdapter>>,
    pub(super) signal_sink: Option<Box<dyn WorthQueryRuntimeSignalSinkAdapter>>,
    pub(super) subscription_activation:
        Option<Box<dyn WorthQueryRuntimeSubscriptionActivationAdapter>>,
    pub(super) preview_basis: Option<Box<dyn WorthQueryRuntimePreviewBasisAdapter>>,
    pub(super) inspector_evidence: Option<Box<dyn WorthQueryRuntimeInspectorEvidenceAdapter>>,
    pub(super) declaration_initialization:
        Option<Box<dyn WorthQueryRuntimeDeclarationInitializationAdapter>>,
    pub(super) intent_authority: Option<Box<dyn WorthQueryIntentAuthorityAdapter>>,
    pub(super) support_profile: Option<WorthQueryRuntimeSupportProfile>,
}

impl WorthQueryRuntimeBackendParts {
    pub fn new() -> Self {
        Self::default()
    }

    pub(in crate::runtime) fn is_empty(&self) -> bool {
        self.relational_runtime.is_none()
            && self.runtime_bridge.is_none()
            && self.schema_adapter.is_none()
            && self.source_adapter.is_none()
            && self.snapshot_identity.is_none()
            && self.existing_truth_verification.is_none()
            && self.write_authority.is_none()
            && self.signal_sink.is_none()
            && self.subscription_activation.is_none()
            && self.preview_basis.is_none()
            && self.inspector_evidence.is_none()
            && self.declaration_initialization.is_none()
            && self.intent_authority.is_none()
            && self.support_profile.is_none()
    }

    pub(in crate::runtime) fn has_relational_runtime(&self) -> bool {
        self.relational_runtime.is_some()
    }

    pub(in crate::runtime) fn lower_bridge_backed_bootstrap(
        self,
    ) -> Result<BridgeBackedRuntimeBootstrap, WorthQueryRuntimeError> {
        BridgeBackedRuntimeBootstrap::lower_from_parts(self)
    }

    pub fn relational_runtime(mut self, runtime: RelationalRuntime) -> Self {
        self.relational_runtime =
            Some(super::relational_owner::WorthQueryBackendRelationalOwner::Unpublished(runtime));
        self
    }

    pub fn relational_source_owner(
        mut self,
        owner: worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
    ) -> Self {
        self.relational_runtime =
            Some(super::relational_owner::WorthQueryBackendRelationalOwner::ProductSource(owner));
        self
    }

    pub fn runtime_bridge(mut self, bridge: RuntimeBridge) -> Self {
        self.runtime_bridge = Some(bridge);
        self
    }

    pub fn schema_adapter(
        mut self,
        adapter: impl WorthQueryRuntimeSchemaAdapter + 'static,
    ) -> Self {
        self.schema_adapter = Some(Box::new(adapter));
        self
    }

    pub fn source_adapter(
        mut self,
        adapter: impl WorthQueryRuntimeSourceAdapter + 'static,
    ) -> Self {
        self.source_adapter = Some(Box::new(adapter));
        self
    }

    pub fn snapshot_identity(
        mut self,
        adapter: impl WorthQueryRuntimeSnapshotIdentityAdapter + 'static,
    ) -> Self {
        self.snapshot_identity = Some(Box::new(adapter));
        self
    }

    pub fn existing_truth_verification(
        mut self,
        adapter: impl WorthQueryRuntimeExistingTruthVerificationAdapter + 'static,
    ) -> Self {
        self.existing_truth_verification = Some(Box::new(adapter));
        self
    }

    pub fn write_authority(
        mut self,
        authority: impl WorthQueryRuntimeWriteAuthorityAdapter + 'static,
    ) -> Self {
        self.write_authority = Some(Box::new(authority));
        self
    }

    pub fn signal_sink(mut self, sink: impl WorthQueryRuntimeSignalSinkAdapter + 'static) -> Self {
        self.signal_sink = Some(Box::new(sink));
        self
    }

    pub fn subscription_activation(
        mut self,
        adapter: impl WorthQueryRuntimeSubscriptionActivationAdapter + 'static,
    ) -> Self {
        self.subscription_activation = Some(Box::new(adapter));
        self
    }

    pub fn preview_basis(
        mut self,
        adapter: impl WorthQueryRuntimePreviewBasisAdapter + 'static,
    ) -> Self {
        self.preview_basis = Some(Box::new(adapter));
        self
    }

    pub fn inspector_evidence(
        mut self,
        adapter: impl WorthQueryRuntimeInspectorEvidenceAdapter + 'static,
    ) -> Self {
        self.inspector_evidence = Some(Box::new(adapter));
        self
    }

    pub fn declaration_initialization(
        mut self,
        adapter: impl WorthQueryRuntimeDeclarationInitializationAdapter + 'static,
    ) -> Self {
        self.declaration_initialization = Some(Box::new(adapter));
        self
    }

    pub fn intent_authority(
        mut self,
        adapter: impl WorthQueryIntentAuthorityAdapter + 'static,
    ) -> Self {
        self.intent_authority = Some(Box::new(adapter));
        self
    }

    pub fn support_profile(mut self, profile: WorthQueryRuntimeSupportProfile) -> Self {
        self.support_profile = Some(profile);
        self
    }
}
