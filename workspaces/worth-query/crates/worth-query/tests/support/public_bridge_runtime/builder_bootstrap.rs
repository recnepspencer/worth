use super::*;

impl PublicBridgeRuntimeHarness {
    pub fn bridge_backed_runtime_builder(&self) -> PublicBridgeRuntimeBootstrapBuilder {
        PublicBridgeRuntimeBootstrapBuilder {
            state: self.state.clone(),
        }
    }

    pub fn configure_runtime_builder(
        &self,
        builder: WorthQueryRuntimeBuilder,
        bridge: worth_runtime_bridge::facade::RuntimeBridge,
        contracts: impl IntoIterator<Item = worth_foundational::facade::AspectContract>,
        support_profile: WorthQueryRuntimeSupportProfile,
    ) -> WorthQueryRuntimeBuilder {
        builder
            .aspect_contracts(contracts)
            .expect("public bridge test contracts should install")
            .runtime_bridge(bridge)
            .schema_adapter(PublicSchemaAdapter)
            .source_adapter(PublicSourceAdapter::new(self.state.clone()))
            .existing_truth_verification(PublicExistingTruthVerificationAdapter::new(
                self.state.clone(),
            ))
            .write_authority(PublicWriteAuthorityAdapter::new(self.state.clone()))
            .snapshot_identity(PublicSnapshotIdentityAdapter::new(self.state.clone()))
            .signal_sink(PublicSignalSinkAdapter)
            .subscription_activation(PublicSubscriptionActivationAdapter)
            .preview_basis(PublicPreviewBasisAdapter)
            .inspector_evidence(PublicInspectorEvidenceAdapter)
            .support_profile(support_profile)
    }

    pub fn advance_snapshot(&self) {
        self.state.borrow_mut().next_snapshot_token += 1;
    }
}

impl PublicBridgeRuntimeBootstrapBuilder {
    pub fn support_profile(
        self,
        support_profile: WorthQueryRuntimeSupportProfile,
    ) -> PublicBridgeRuntimeBootstrapWithSupportProfile {
        PublicBridgeRuntimeBootstrapWithSupportProfile {
            state: self.state,
            support_profile,
        }
    }
}

impl PublicBridgeRuntimeBootstrapWithSupportProfile {
    pub fn build(self) -> WorthQueryRuntime {
        record_public_bridge_runtime_bootstrap_invocation(
            PublicBridgeRuntimeBootstrapPath::Builder,
        );

        WorthQueryRuntime::builder()
            .aspect_contracts(public_bridge_aspect_contracts())
            .expect("public bridge aspect contracts should install")
            .runtime_bridge(bridge::public_bridge())
            .schema_adapter(PublicSchemaAdapter)
            .source_adapter(PublicSourceAdapter::new(self.state.clone()))
            .existing_truth_verification(PublicExistingTruthVerificationAdapter::new(
                self.state.clone(),
            ))
            .write_authority(PublicWriteAuthorityAdapter::new(self.state.clone()))
            .snapshot_identity(PublicSnapshotIdentityAdapter::new(self.state))
            .signal_sink(PublicSignalSinkAdapter)
            .subscription_activation(PublicSubscriptionActivationAdapter)
            .preview_basis(PublicPreviewBasisAdapter)
            .inspector_evidence(PublicInspectorEvidenceAdapter)
            .support_profile(self.support_profile)
            .build_backend_from_parts()
            .build()
            .expect("public bridge-backed runtime should build")
    }
}
