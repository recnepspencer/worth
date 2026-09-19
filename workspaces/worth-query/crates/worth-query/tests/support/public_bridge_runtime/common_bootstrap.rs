use super::*;

impl PublicBridgeRuntimeHarness {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(PublicBridgeRuntimeState::default())),
        }
    }
    pub fn bridge_backed_runtime(&self) -> WorthQueryRuntime {
        self.bridge_backed_runtime_with_support(public_graph_support_profile())
    }

    pub fn bridge_backed_runtime_with_support(
        &self,
        profile: WorthQueryRuntimeSupportProfile,
    ) -> WorthQueryRuntime {
        self.bridge_backed_runtime_with_support_and_relational(profile, None)
    }

    pub fn bridge_backed_runtime_with_relational(
        &self,
        relational_runtime: worth_relational::facade::runtime::RelationalRuntime,
    ) -> WorthQueryRuntime {
        self.bridge_backed_runtime_with_support_and_relational(
            public_graph_support_profile(),
            Some(relational_runtime),
        )
    }

    fn bridge_backed_runtime_with_support_and_relational(
        &self,
        profile: WorthQueryRuntimeSupportProfile,
        relational_runtime: Option<worth_relational::facade::runtime::RelationalRuntime>,
    ) -> WorthQueryRuntime {
        record_public_bridge_runtime_bootstrap_invocation(PublicBridgeRuntimeBootstrapPath::Common);
        let relational_runtime = relational_runtime.unwrap_or_else(|| {
            worth_relational::facade::runtime::RelationalRuntimeBuilder::new().build()
        });
        let source =
            worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
                relational_runtime,
                "public-graph",
            )
            .expect("public bridge tests require one real Relational product source");
        let bridge = bridge::public_bridge(&source);

        WorthQueryRuntime::builder(public_product_world_resources())
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
            .snapshot_identity(PublicSnapshotIdentityAdapter::new(self.state.clone()))
            .signal_sink(PublicSignalSinkAdapter)
            .subscription_activation(PublicSubscriptionActivationAdapter)
            .preview_basis(PublicPreviewBasisAdapter)
            .inspector_evidence(PublicInspectorEvidenceAdapter)
            .support_profile(profile)
            .build_backend_from_parts()
            .build()
            .expect("public bridge-backed runtime should build")
    }
    pub fn seed_backend_authoritative_truth(
        &self,
        binding: &WorthQueryExistingTruthTargetBinding,
        aspect_touch: WorthQueryAspectTouch,
        value: AspectValue,
    ) -> PublicExistingTruthSeedRecord {
        let record = PublicExistingTruthSeedRecord::new(binding, aspect_touch);
        self.state
            .borrow_mut()
            .existing_truth_values
            .insert(record.key.clone(), value);
        record
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicExistingTruthSeedRecord {
    key: PublicExistingTruthKey,
}

impl PublicExistingTruthSeedRecord {
    fn new(
        binding: &WorthQueryExistingTruthTargetBinding,
        aspect_touch: WorthQueryAspectTouch,
    ) -> Self {
        Self {
            key: PublicExistingTruthKey::new(binding, aspect_touch),
        }
    }
    pub fn binding_digest(&self) -> &str {
        self.key.binding_digest()
    }
    pub fn target_collection(&self) -> &str {
        self.key.target_collection()
    }
    pub fn admitted_aspect_touch_reporting_projection(&self) -> String {
        self.key.admitted_aspect_touch_reporting_projection()
    }
}
pub fn reset_public_bridge_runtime_bootstrap_invocations() {
    BOOTSTRAP_INVOCATIONS.with(|counts| {
        *counts.borrow_mut() = [0; 2];
    });
}
pub fn public_bridge_runtime_bootstrap_invocation_count(
    path: PublicBridgeRuntimeBootstrapPath,
) -> usize {
    BOOTSTRAP_INVOCATIONS.with(|counts| counts.borrow()[bootstrap_index(path)])
}
