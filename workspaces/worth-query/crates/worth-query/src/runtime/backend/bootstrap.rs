use worth_runtime_bridge::facade::RuntimeBridge;

use crate::runtime::{WorthQueryRuntimeError, WorthQueryRuntimeSupportProfile};

use super::{
    WorthQueryIntentAuthorityAdapter, WorthQueryRuntimeBackendParts,
    WorthQueryRuntimeDeclarationInitializationAdapter,
    WorthQueryRuntimeExistingTruthVerificationAdapter, WorthQueryRuntimeInspectorEvidenceAdapter,
    WorthQueryRuntimePreviewBasisAdapter, WorthQueryRuntimeSchemaAdapter,
    WorthQueryRuntimeSignalSinkAdapter, WorthQueryRuntimeSnapshotIdentityAdapter,
    WorthQueryRuntimeSourceAdapter, WorthQueryRuntimeSubscriptionActivationAdapter,
    WorthQueryRuntimeWriteAuthorityAdapter,
};

pub(in crate::runtime) struct BridgeBackedRuntimeBootstrap {
    pub(super) relational_runtime:
        Option<super::relational_owner::WorthQueryBackendRelationalOwner>,
    pub(super) runtime_bridge: RuntimeBridge,
    pub(super) schema_adapter: Box<dyn WorthQueryRuntimeSchemaAdapter>,
    pub(super) source_adapter: Box<dyn WorthQueryRuntimeSourceAdapter>,
    pub(super) snapshot_identity: Option<Box<dyn WorthQueryRuntimeSnapshotIdentityAdapter>>,
    pub(super) existing_truth_verification:
        Option<Box<dyn WorthQueryRuntimeExistingTruthVerificationAdapter>>,
    pub(super) write_authority: Box<dyn WorthQueryRuntimeWriteAuthorityAdapter>,
    pub(super) signal_sink: Box<dyn WorthQueryRuntimeSignalSinkAdapter>,
    pub(super) subscription_activation: Box<dyn WorthQueryRuntimeSubscriptionActivationAdapter>,
    pub(super) preview_basis: Box<dyn WorthQueryRuntimePreviewBasisAdapter>,
    pub(super) inspector_evidence: Box<dyn WorthQueryRuntimeInspectorEvidenceAdapter>,
    pub(super) declaration_initialization:
        Option<Box<dyn WorthQueryRuntimeDeclarationInitializationAdapter>>,
    pub(super) intent_authority: Option<Box<dyn WorthQueryIntentAuthorityAdapter>>,
    pub(super) support_profile: WorthQueryRuntimeSupportProfile,
}

impl BridgeBackedRuntimeBootstrap {
    pub(super) fn lower_from_parts(
        parts: WorthQueryRuntimeBackendParts,
    ) -> Result<Self, WorthQueryRuntimeError> {
        let relational_runtime = parts.relational_runtime;
        let runtime_bridge = parts
            .runtime_bridge
            .ok_or(WorthQueryRuntimeError::MissingRuntimeBridge)?;
        let schema_adapter = parts
            .schema_adapter
            .ok_or(WorthQueryRuntimeError::MissingSchemaAdapter)?;
        let source_adapter = parts
            .source_adapter
            .ok_or(WorthQueryRuntimeError::MissingSourceAdapter)?;
        let snapshot_identity = parts
            .snapshot_identity
            .ok_or(WorthQueryRuntimeError::MissingSnapshotIdentityAdapter)?;
        let existing_truth_verification = parts.existing_truth_verification;
        let write_authority = parts
            .write_authority
            .ok_or(WorthQueryRuntimeError::MissingWriteAuthority)?;
        let signal_sink = parts
            .signal_sink
            .ok_or(WorthQueryRuntimeError::MissingSignalSink)?;
        let subscription_activation = parts
            .subscription_activation
            .ok_or(WorthQueryRuntimeError::MissingSubscriptionActivation)?;
        let preview_basis = parts
            .preview_basis
            .ok_or(WorthQueryRuntimeError::MissingPreviewBasis)?;
        let inspector_evidence = parts
            .inspector_evidence
            .ok_or(WorthQueryRuntimeError::MissingInspectorEvidence)?;
        let declaration_initialization = parts.declaration_initialization;
        let intent_authority = parts.intent_authority;
        let support_profile = parts.support_profile.unwrap_or_else(|| {
            WorthQueryRuntimeSupportProfile::bridge_backed(
                subscription_activation.support_evidence_for_reporting(),
                "preview-basis-admission",
                "inspector-evidence-adapter",
            )
        });

        support_profile
            .validate_backend_claims(intent_authority.is_some())
            .map_err(WorthQueryRuntimeError::UnsupportedFacadeFamily)?;

        Ok(Self {
            relational_runtime,
            runtime_bridge,
            schema_adapter,
            source_adapter,
            snapshot_identity: Some(snapshot_identity),
            existing_truth_verification,
            write_authority,
            signal_sink,
            subscription_activation,
            preview_basis,
            inspector_evidence,
            declaration_initialization,
            intent_authority,
            support_profile,
        })
    }
}
