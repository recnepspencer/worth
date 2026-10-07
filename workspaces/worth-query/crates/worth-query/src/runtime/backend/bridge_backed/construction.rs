use super::*;

impl WorthQueryBridgeBackedRuntimeBackend {
    #[cfg(test)]
    pub(crate) fn from_parts(
        parts: super::super::WorthQueryRuntimeBackendParts,
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
