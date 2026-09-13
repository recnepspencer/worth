use std::collections::BTreeSet;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey};
use worth_query::facade::foundation::{
    DeclarativeLiveQueryRequest, WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle,
    WorthQueryMutationReceipt, WorthQuerySnapshotIdentity, WorthQueryWorkspaceError,
};
use worth_query::facade::runtime::{
    LiveViewDeclarationAdmissionBoundaryReceipt, QuerySchemaView, SubscriptionActivationInput,
    SubscriptionActivationReceipt, WorthQueryBackendAdmissibleMutation,
    WorthQueryIntentDeclaration, WorthQueryIntentExecution, WorthQueryLiveArtifactTarget,
    WorthQueryPreviewBasisAdmission, WorthQueryRuntime, WorthQueryRuntimeBackend,
    WorthQueryRuntimeError, WorthQueryRuntimeEvidenceAuthority,
    WorthQueryRuntimeInspectionEvidence, WorthQueryRuntimeSchemaAdapter,
    WorthQueryRuntimeSubscriptionActivationAdapter, WorthQueryRuntimeSupportProfile,
    WorthQuerySessionLabel, WorthQueryWorkspace, WorthQueryWriteReceipt,
};
use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;
use worth_server::{
    WorthServerQueryWorkspaceBindingError, WorthServerQueryWorkspaceBindingRequest,
    WorthServerQueryWorkspaceProvider,
};

use super::runtime_aspect_contracts::query_handoff_aspect_contracts;
use super::runtime_mutation_support::{test_mutation_receipt, TestSubscriptionActivation};
use super::runtime_named_read::install_requested_named_read;

#[derive(Clone, Debug, Default)]
pub(crate) struct TestWorkspaceProvider;

impl WorthServerQueryWorkspaceProvider for TestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        bind_with_backend(TestQueryRuntimeBackend::default(), request)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProfiledTestWorkspaceProvider {
    support_profile: WorthQueryRuntimeSupportProfile,
}

impl ProfiledTestWorkspaceProvider {
    pub(crate) fn new(support_profile: WorthQueryRuntimeSupportProfile) -> Self {
        Self { support_profile }
    }
}

impl WorthServerQueryWorkspaceProvider for ProfiledTestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "profiled-test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        bind_with_backend(
            TestQueryRuntimeBackend::new(self.support_profile.clone()),
            request,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProfiledCountingTestWorkspaceProvider {
    support_profile: WorthQueryRuntimeSupportProfile,
    attempted_writes: Arc<AtomicUsize>,
}

impl ProfiledCountingTestWorkspaceProvider {
    pub(crate) fn new(
        support_profile: WorthQueryRuntimeSupportProfile,
        attempted_writes: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            support_profile,
            attempted_writes,
        }
    }
}

impl WorthServerQueryWorkspaceProvider for ProfiledCountingTestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "profiled-counting-test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        bind_with_backend(
            TestQueryRuntimeBackend::new_with_attempted_writes(
                self.support_profile.clone(),
                self.attempted_writes.clone(),
            ),
            request,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PanicOnReadTestWorkspaceProvider;

impl WorthServerQueryWorkspaceProvider for PanicOnReadTestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "panic-on-read-test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        bind_with_backend(
            TestQueryRuntimeBackend::new_panicking_on_live_reads(),
            request,
        )
    }
}

fn bind_with_backend(
    backend: TestQueryRuntimeBackend,
    request: &WorthServerQueryWorkspaceBindingRequest,
) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
    let workspace_id = request
        .resolved_request_context()
        .request_context()
        .workspace_target()
        .workspace_id();
    let mut workspace = WorthQueryRuntime::builder(
        worth_query::facade::consumer_kit::in_memory_test_product_world_resources(),
    )
    .aspect_contracts(query_handoff_aspect_contracts())
    .map_err(|error| {
        WorthServerQueryWorkspaceBindingError::new(
            "aspect_contracts",
            format!("failed to install query handoff aspect contracts: {error}"),
        )
    })?
    .backend(backend)
    .build()
    .map_err(|error| {
        WorthServerQueryWorkspaceBindingError::new("runtime_build", format!("{error:?}"))
    })?
    .workspace(workspace_id)
    .map_err(|error| {
        WorthServerQueryWorkspaceBindingError::new("workspace_bind", format!("{error:?}"))
    })?;
    install_requested_named_read(&mut workspace, request)?;
    Ok(workspace)
}

#[derive(Clone, Debug)]
struct TestQueryRuntimeBackend {
    support_profile: WorthQueryRuntimeSupportProfile,
    declared_live_views: BTreeSet<String>,
    attempted_writes: Option<Arc<AtomicUsize>>,
    panic_on_live_reads: bool,
}

impl Default for TestQueryRuntimeBackend {
    fn default() -> Self {
        Self::new(WorthQueryRuntimeSupportProfile::scaffold_backend_profile())
    }
}

impl TestQueryRuntimeBackend {
    fn new(support_profile: WorthQueryRuntimeSupportProfile) -> Self {
        Self {
            support_profile,
            declared_live_views: BTreeSet::new(),
            attempted_writes: None,
            panic_on_live_reads: false,
        }
    }

    fn new_with_attempted_writes(
        support_profile: WorthQueryRuntimeSupportProfile,
        attempted_writes: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            support_profile,
            declared_live_views: BTreeSet::new(),
            attempted_writes: Some(attempted_writes),
            panic_on_live_reads: false,
        }
    }

    fn new_panicking_on_live_reads() -> Self {
        Self {
            support_profile: WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            declared_live_views: BTreeSet::new(),
            attempted_writes: None,
            panic_on_live_reads: true,
        }
    }

    fn record_attempted_write(&self, count: usize) {
        if let Some(attempted_writes) = &self.attempted_writes {
            attempted_writes.fetch_add(count, Ordering::Relaxed);
        }
    }
}

impl worth_query::facade::runtime::WorthQuerySettlementRecoveryBackend for TestQueryRuntimeBackend {}

impl worth_query::facade::runtime::WorthQueryMergeSnapshotOwner for TestQueryRuntimeBackend {}

impl WorthQueryRuntimeBackend for TestQueryRuntimeBackend {
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
        Ok(WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError> {
        self.declared_live_views.remove(name);
        Ok(())
    }

    fn write(
        &mut self,
        command: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        self.record_attempted_write(1);
        Ok(test_mutation_receipt(&command, 1))
    }

    fn write_batch(
        &mut self,
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
        _declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryRuntimeError> {
        panic!("unused in query handoff phase tests")
    }

    fn live_entities_for_target(
        &self,
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

        vec![WorthQueryEntity::from_native_field_values(
            worth_query::facade::foundation::WorthQueryEntityIdentity::admit_authored_entity_token(
                worth_query::facade::foundation::QueryExternalIdentityToken::new(
                    std::sync::Arc::from("user-1"),
                ),
            ),
            std::collections::BTreeMap::from([
                (
                    field_path("identity.id"),
                    AspectValue::String("user-1".into()),
                ),
                (
                    field_path("profile.display_name"),
                    AspectValue::String("Ada Worth".into()),
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

    fn current_snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(
            worth_runtime_bridge::facade::TruthSnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(1, 1),
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
        Ok(WorthQueryRuntimeInspectionEvidence::new(
            authority,
            "query-handoff-phase-test-write-receipt",
            receipt.authority_lane(),
            ["query-handoff-phase-test-inspector"],
        ))
    }
}

struct TestSchemaAdapter;

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

fn field_path(path: &str) -> CanonicalFieldPath {
    let fields = path
        .split('.')
        .map(|field| {
            FieldKey::new(field).expect("synthetic runtime field segments should be foundational")
        })
        .collect::<Vec<_>>();
    CanonicalFieldPath::new(fields).expect("synthetic runtime field path should be non-empty")
}
