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
        self.build_with_product_world_resources(public_product_world_resources())
    }

    pub fn build_with_product_world_resources(
        self,
        product_world_resources: WorthQueryProductWorldResources,
    ) -> WorthQueryRuntime {
        record_public_bridge_runtime_bootstrap_invocation(
            PublicBridgeRuntimeBootstrapPath::Builder,
        );
        let source =
            worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
                worth_relational::facade::runtime::RelationalRuntimeBuilder::new().build(),
                "public-graph",
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
            .build()
            .expect("public bridge-backed runtime should build")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_runtime_requires_explicit_product_resources() {
        let harness = PublicBridgeRuntimeHarness::new();
        let source = product_source("missing-resources");
        let bridge = bridge::public_bridge(&source);
        let result = harness
            .configure_runtime_builder(
                WorthQueryRuntime::builder(public_product_world_resources())
                    .relational_source_owner(source),
                bridge,
                public_bridge_aspect_contracts(),
                public_graph_support_profile(),
            )
            .build_backend_from_parts()
            .build();

        let error = match result {
            Ok(_) => panic!("execution runtime must require Product World budgets"),
            Err(error) => error,
        };
        assert!(format!("{error:?}").contains("conditional_execution_resources"));
    }

    #[test]
    fn execution_runtime_requires_an_installed_relational_product_source() {
        let harness = PublicBridgeRuntimeHarness::new();
        let bridge_source = product_source("bridge-only-source");
        let bridge = bridge::public_bridge(&bridge_source);
        let result = harness
            .configure_runtime_builder(
                WorthQueryRuntime::builder(public_product_world_resources())
                    .conditional_execution_resources(public_product_resources()),
                bridge,
                public_bridge_aspect_contracts(),
                public_graph_support_profile(),
            )
            .build_backend_from_parts()
            .build();

        let error = match result {
            Ok(_) => panic!("execution runtime must require its Relational owner root"),
            Err(error) => error,
        };
        assert!(format!("{error:?}").contains("SourceNotInstalled"));
    }

    #[test]
    fn execution_runtime_rejects_a_bridge_from_another_relational_owner() {
        let harness = PublicBridgeRuntimeHarness::new();
        let installed_source = product_source("product-source");
        let foreign_bridge_source = product_source("foreign-bridge-source");
        let bridge = bridge::public_bridge(&foreign_bridge_source);
        let result = harness
            .configure_runtime_builder(
                WorthQueryRuntime::builder(public_product_world_resources())
                    .relational_source_owner(installed_source)
                    .conditional_execution_resources(public_product_resources()),
                bridge,
                public_bridge_aspect_contracts(),
                public_graph_support_profile(),
            )
            .build_backend_from_parts()
            .build();

        let error = match result {
            Ok(_) => panic!("foreign Relational/Bridge pairing must fail closed"),
            Err(error) => error,
        };
        assert!(format!("{error:?}").contains("source"));
    }

    fn product_source(
        role: &'static str,
    ) -> worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner {
        worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
            worth_relational::facade::runtime::RelationalRuntimeBuilder::new().build(),
            role,
        )
        .expect("test Product World source should install")
    }
}
