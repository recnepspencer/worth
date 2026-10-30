pub use {std::collections::BTreeMap, std::collections::BTreeSet};
pub use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

pub use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;
pub use {
    worth_foundational::facade::AspectValue, worth_foundational::facade::CanonicalFieldPath,
    worth_foundational::facade::FieldKey,
};
pub use {
    worth_query::facade::foundation::DeclarativeLiveQueryRequest,
    worth_query::facade::foundation::QueryExternalIdentityToken,
    worth_query::facade::foundation::WorthQueryEntity,
    worth_query::facade::foundation::WorthQueryEntityIdentity,
    worth_query::facade::foundation::WorthQueryLivePatch,
    worth_query::facade::foundation::WorthQueryLiveViewHandle,
    worth_query::facade::foundation::WorthQueryMutationReceipt,
    worth_query::facade::foundation::WorthQuerySnapshotIdentity,
    worth_query::facade::foundation::WorthQueryWorkspaceError,
};
pub use {
    worth_query::facade::runtime::LiveViewDeclarationAdmissionBoundaryReceipt,
    worth_query::facade::runtime::QuerySchemaView,
    worth_query::facade::runtime::SubscriptionActivationInput,
    worth_query::facade::runtime::SubscriptionActivationReceipt,
    worth_query::facade::runtime::WorthQueryBackendAdmissibleMutation,
    worth_query::facade::runtime::WorthQueryIntentDeclaration,
    worth_query::facade::runtime::WorthQueryIntentExecution,
    worth_query::facade::runtime::WorthQueryLiveArtifactTarget,
    worth_query::facade::runtime::WorthQueryPreviewBasisAdmission,
    worth_query::facade::runtime::WorthQueryRuntimeBackend,
    worth_query::facade::runtime::WorthQueryRuntimeError,
    worth_query::facade::runtime::WorthQueryRuntimeEvidenceAuthority,
    worth_query::facade::runtime::WorthQueryRuntimeInspectionEvidence,
    worth_query::facade::runtime::WorthQueryRuntimeSchemaAdapter,
    worth_query::facade::runtime::WorthQueryRuntimeSubscriptionActivationAdapter,
    worth_query::facade::runtime::WorthQueryRuntimeSupportProfile,
    worth_query::facade::runtime::WorthQuerySessionLabel,
    worth_query::facade::runtime::WorthQueryWriteReceipt,
};

pub use {
    crate::shared_support_query_handoff_runtime::runtime_mutation_support::test_mutation_receipt,
    crate::shared_support_query_handoff_runtime::runtime_mutation_support::TestSubscriptionActivation,
};

#[derive(Clone)]
pub struct TestQueryRuntimeBackend {
    pub support_profile: WorthQueryRuntimeSupportProfile,
    pub declared_live_views: BTreeSet<String>,
    pub attempted_writes: Option<Arc<AtomicUsize>>,
    pub panic_on_live_reads: bool,
    pub product_source: Option<worth_query::facade::runtime::WorthQueryRelationalSourceOwner>,
}

impl Default for TestQueryRuntimeBackend {
    fn default() -> Self {
        Self::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(
            ),
        )
    }
}

impl TestQueryRuntimeBackend {
    pub fn new(support_profile: WorthQueryRuntimeSupportProfile) -> Self {
        Self {
            support_profile,
            declared_live_views: std::collections::BTreeSet::new(),
            attempted_writes: None,
            panic_on_live_reads: false,
            product_source: None,
        }
    }

    pub fn new_with_attempted_writes(
        support_profile: WorthQueryRuntimeSupportProfile,
        attempted_writes: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            support_profile,
            declared_live_views: std::collections::BTreeSet::new(),
            attempted_writes: Some(attempted_writes),
            panic_on_live_reads: false,
            product_source: None,
        }
    }

    pub fn new_panicking_on_live_reads() -> Self {
        Self {
            support_profile: worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            declared_live_views: std::collections::BTreeSet::new(),
            attempted_writes: None,
            panic_on_live_reads: true,
            product_source: None,
        }
    }

    pub fn with_product_source(
        mut self,
        product_source: worth_query::facade::runtime::WorthQueryRelationalSourceOwner,
    ) -> Self {
        self.product_source = Some(product_source);
        self
    }

    pub fn record_attempted_write(&self, count: usize) {
        if let Some(attempted_writes) = &self.attempted_writes {
            attempted_writes.fetch_add(count, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

impl worth_query::facade::runtime::WorthQuerySettlementRecoveryBackend for TestQueryRuntimeBackend {}

impl worth_query::facade::runtime::WorthQueryMergeSnapshotOwner for TestQueryRuntimeBackend {}

impl WorthQueryRuntimeBackend for TestQueryRuntimeBackend {
    fn prepare_product_source(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryProductRelationalInstallation,
        worth_query::facade::runtime::WorthQueryProductSourceDenial,
    > {
        let source = self
            .product_source
            .as_ref()
            .ok_or(worth_query::facade::runtime::WorthQueryProductSourceDenial::Unsupported)?;
        let branch =
            source.with_runtime(request_execution, |runtime| runtime.main_branch_identity());
        source
            .prepare_product_source(request_execution, &branch)
            .map_err(worth_query::facade::runtime::WorthQueryProductSourceDenial::Basis)
    }

    fn support_profile(&self) -> WorthQueryRuntimeSupportProfile {
        self.support_profile.clone()
    }

    fn admit_live_view_declaration(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError> {
        TestSchemaAdapter.admit_live_view(name, request, schema_view)
    }

    fn declare_live_view(
        &mut self,
        name: String,
        _request: DeclarativeLiveQueryRequest,
        _schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        self.declared_live_views.insert(name.clone());
        Ok(worth_query::facade::foundation::WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError> {
        self.declared_live_views.remove(name);
        Ok(())
    }

    fn write(
        &mut self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        command: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        self.record_attempted_write(1);
        Ok(test_mutation_receipt(&command, 1))
    }

    fn write_batch(
        &mut self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        commands: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<Vec<WorthQueryMutationReceipt>, WorthQueryWorkspaceError> {
        self.record_attempted_write(commands.len());
        Ok(commands
            .iter()
            .enumerate()
            .map(|(index, command)| test_mutation_receipt(command, index + 1))
            .collect())
    }

    fn execute_intent(
        &mut self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        _declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryRuntimeError> {
        panic!("unused in query handoff phase tests")
    }

    fn live_entities_for_target(
        &self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryEntity> {
        assert!(
            !self.panic_on_live_reads,
            "live entity reads must not execute for this hostile denial seam"
        );
        if !self
            .declared_live_views
            .contains(target.terminal_view_name_projection())
        {
            return Vec::new();
        }

        vec![worth_query::facade::foundation::WorthQueryEntity::from_native_field_values(
            worth_query::facade::foundation::WorthQueryEntityIdentity::admit_authored_entity_token(worth_query::facade::foundation::QueryExternalIdentityToken::new(
                std::sync::Arc::from("user-1"),
            )),
            std::collections::BTreeMap::from([
                (
                    field_path("identity.id"),
                    worth_foundational::facade::AspectValue::String("user-1".into()),
                ),
                (
                    field_path("profile.display_name"),
                    worth_foundational::facade::AspectValue::String("Ada Worth".into()),
                ),
            ]),
        )]
    }

    fn drain_live_patches_for_target(
        &mut self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        Vec::new()
    }

    fn affected_live_view_targets(
        &self,
        _receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        Vec::new()
    }

    fn current_snapshot_identity(
        &self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQuerySnapshotIdentity {
        worth_query::facade::foundation::WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(
            worth_runtime_bridge::facade::TruthSnapshotIdentity::from_relational_snapshot(
                worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts::new(1, 1),
            ),
        )
        .expect("relational snapshot projection must retain its typed payload")
    }

    fn install_live_subscription(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationReceipt, WorthQueryWorkspaceError> {
        let mut activation_adapter = TestSubscriptionActivation;
        let receipt = activation_adapter.admit_activation(view_name, activation)?;
        Ok(receipt.activation_receipt().clone())
    }

    fn admit_preview_basis(
        &self,
        _label: &WorthQuerySessionLabel,
        _effect_policy: worth_query::facade::runtime::WorthQueryEffectPolicy,
        _authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError> {
        panic!("unused in query handoff phase tests")
    }

    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError> {
        Ok(
            worth_query::facade::runtime::WorthQueryRuntimeInspectionEvidence::new(
                authority,
                "query-handoff-phase-test-write-receipt",
                receipt.authority_lane(),
                ["query-handoff-phase-test-inspector"],
            ),
        )
    }
}

pub struct TestSchemaAdapter;

impl WorthQueryRuntimeSchemaAdapter for TestSchemaAdapter {
    fn admit_live_view(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        _schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError> {
        let admission = self.build_live_view_declaration_admission_receipt(name, request);
        Ok(self.build_live_view_declaration_boundary_receipt(name, request, admission))
    }
}

pub fn field_path(path: &str) -> CanonicalFieldPath {
    let fields = path
        .split('.')
        .map(|field| {
            worth_foundational::facade::FieldKey::new(field)
                .expect("synthetic runtime field segments should be foundational")
        })
        .collect::<Vec<_>>();
    worth_foundational::facade::CanonicalFieldPath::new(fields)
        .expect("synthetic runtime field path should be non-empty")
}
