pub use super::*;

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
    pub fn build(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQueryRuntime {
        self.build_with_product_world_resources(request_execution, public_product_world_resources())
    }

    pub fn build_with_product_world_resources(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        product_world_resources: WorthQueryProductWorldResources,
    ) -> WorthQueryRuntime {
        record_public_bridge_runtime_bootstrap_invocation(
            PublicBridgeRuntimeBootstrapPath::Builder,
        );
        let source =
            worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
                worth_relational::facade::runtime::RelationalRuntimeBuilder::new()
                    .build()
                    .expect("valid native runtime configuration"),
                "public-graph",
                product_world_resources.invalidation_resources(),
            )
            .expect("public bridge tests require one real Relational product source");
        let bridge = bridge::public_bridge(&source);

        WorthQueryRuntime::builder(product_world_resources)
            .aspect_contracts(public_bridge_aspect_contracts())
            .expect("public bridge aspect contracts should install")
            .relational_source_owner(source)
            .runtime_bridge(bridge)
            .conditional_execution_resources(public_product_resources())
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
            .build(request_execution)
            .expect("public bridge-backed runtime should build")
    }
}

pub mod tests {
    pub use super::*;

    pub fn product_source(
        role: &'static str,
        resources: &WorthQueryProductWorldResources,
    ) -> worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner {
        worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
            worth_relational::facade::runtime::RelationalRuntimeBuilder::new()
                .build()
                .expect("valid native runtime configuration"),
            role,
            resources.invalidation_resources(),
        )
        .expect("test Product World source should install")
    }
}
