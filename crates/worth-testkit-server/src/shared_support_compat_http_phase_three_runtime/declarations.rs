

pub use {serde_json::json, serde_json::Value};
pub use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};
pub use {worth_proof::TransitionOutcome};
pub use {worth_query::facade::foundation::WorthQueryCommitIdentity, worth_query::facade::foundation::WorthQueryEntityIdentity, worth_query::facade::foundation::WorthQueryMutationDelta, worth_query::facade::foundation::WorthQueryMutationKind};
pub use {worth_query::facade::foundation::WorthQueryEntity, worth_query::facade::foundation::WorthQueryLivePatch, worth_query::facade::foundation::WorthQueryMutationReceipt, worth_query::facade::foundation::WorthQuerySnapshotIdentity, worth_query::facade::foundation::WorthQueryWorkspaceError};
pub use {worth_query::facade::runtime::SubscriptionActivationInput, worth_query::facade::runtime::SubscriptionActivationReceipt, worth_query::facade::runtime::WorthQueryBackendAdmissibleMutation, worth_query::facade::runtime::WorthQueryIntentDeclaration, worth_query::facade::runtime::WorthQueryIntentExecution, worth_query::facade::runtime::WorthQueryLiveArtifactTarget, worth_query::facade::runtime::WorthQueryPreviewBasisAdmission, worth_query::facade::runtime::WorthQueryRuntime, worth_query::facade::runtime::WorthQueryRuntimeBackend, worth_query::facade::runtime::WorthQueryRuntimeError, worth_query::facade::runtime::WorthQueryRuntimeEvidenceAuthority, worth_query::facade::runtime::WorthQueryRuntimeInspectionEvidence, worth_query::facade::runtime::WorthQueryRuntimeSupportProfile, worth_query::facade::runtime::WorthQuerySessionLabel, worth_query::facade::runtime::WorthQueryWorkspace, worth_query::facade::runtime::WorthQueryWriteCommand, worth_query::facade::runtime::WorthQueryWriteReceipt};
pub use {worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts, worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts};
pub use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerCompatibilityMutationExecutionInput, worth_server::WorthServerCompatibilityPreparedRequest, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerConfig, worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerQueryHandoffConfig, worth_server::WorthServerQueryWorkspaceBindingError, worth_server::WorthServerQueryWorkspaceBindingRequest, worth_server::WorthServerQueryWorkspaceProvider, worth_server::WorthServerRequestContextConfig};

pub use {crate::shared_support_query_handoff_runtime::runtime_aspect_contracts::query_handoff_aspect_contracts};

pub fn build_phase_three_server() -> WorthServer {
    build_phase_three_server_with_workspace_provider(
        crate::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider,
    )
}

pub fn build_phase_three_server_with_workspace_provider(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(worth_server::request_context::DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
}

pub fn mutation_input(
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Mutation)
        .with_method("POST")
        .with_path(format!("/compat/mutations/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type("application/json")
        .with_body_present(true)
}

pub fn prepared_mutation_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility mutation request, got {other:?}"),
    }
}

pub fn compat_mutation_execution_input(
    server: &WorthServer,
    operation_name: &str,
    body: Value,
) -> WorthServerCompatibilityMutationExecutionInput {
    worth_server::WorthServerCompatibilityMutationExecutionInput::new(
        prepared_mutation_request(
            server,
            mutation_input(operation_name)
                .build()
                .expect("compat mutation input should validate structurally"),
        ),
        operation_name,
        body,
    )
}

pub fn single_insert_body(identity: &str) -> Value {
    json!({
        "command": {
            "family": "insert",
            "collection": "Task",
            "aspects": {
                "identity.id": identity,
                "title.value": format!("Title for {identity}")
            }
        }
    })
}

pub fn mutation_request_input_for_workspace(
    operation_name: &str,
    workspace_id: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id(workspace_id)
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Mutation)
        .with_method("POST")
        .with_path(format!("/compat/mutations/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type("application/json")
        .with_body_present(true)
}

pub fn insert_task(identity: &str) -> WorthQueryWriteCommand {
    worth_query::facade::runtime::WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", identity)
        .aspect("title.value", format!("Title for {identity}"))
        .build_insert("Task")
        .expect("insert command should build")
}

pub fn compat_mutation_success(
    outcome: worth_server::WorthServerCompatibilityMutationOutcome<
        worth_server::WorthServerCompatibilityMutation,
    >,
) -> worth_server::WorthServerCompatibilityMutation {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility mutation success, got {other:?}"),
    }
}

pub fn compat_mutation_denied(
    outcome: worth_server::WorthServerCompatibilityMutationOutcome<
        worth_server::WorthServerCompatibilityMutation,
    >,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility mutation denial, got {other:?}"),
    }
}

pub fn direct_mutation_success(
    outcome: worth_server::WorthServerDirectMutationOutcome,
) -> worth_server::WorthServerDirectMutation {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected direct mutation success, got {other:?}"),
    }
}

#[derive(Clone, Debug)]
pub struct StatefulCountingMutationWorkspaceProvider {
    pub support_profile: WorthQueryRuntimeSupportProfile,
    pub attempted_writes: Arc<AtomicUsize>,
    pub snapshot_version: Arc<AtomicUsize>,
}

impl StatefulCountingMutationWorkspaceProvider {
    pub fn new(
        support_profile: WorthQueryRuntimeSupportProfile,
        attempted_writes: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            support_profile,
            attempted_writes,
            snapshot_version: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}

impl WorthServerQueryWorkspaceProvider for StatefulCountingMutationWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "stateful-counting-mutation-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        crate::shared_support_test_bootstrap_policy::with_bootstrap(|request_execution| {
            let workspace_id = request
                .resolved_request_context()
                .request_context()
                .workspace_target()
                .workspace_id();
            let resources =
                worth_query::facade::consumer_kit::in_memory_test_product_world_resources();
            let (product_source, product_bridge) =
                worth_query::facade::consumer_kit::in_memory_test_product_world_installation(
                    &resources,
                )
                .map_err(|error| {
                    worth_server::WorthServerQueryWorkspaceBindingError::new("product_world", error.to_string())
                })?;
            worth_query::facade::runtime::WorthQueryRuntime::builder(resources)
            .aspect_contracts(query_handoff_aspect_contracts())
            .map_err(|error| {
                worth_server::WorthServerQueryWorkspaceBindingError::new(
                    "aspect_contracts",
                    format!("failed to install compatibility mutation aspect contracts: {error}"),
                )
            })?
            .backend(crate::shared_support_compat_http_phase_three_runtime::StatefulCountingMutationRuntimeBackend::new(
                self.support_profile.clone(),
                self.attempted_writes.clone(),
                self.snapshot_version.clone(),
                product_source,
            ))
            .installed_product_bridge(
                product_bridge,
                worth_query::facade::runtime::WorthQueryConditionalExecutionResources::development(
                ),
            )
            .build(request_execution)
            .map_err(|error| {
                worth_server::WorthServerQueryWorkspaceBindingError::new("runtime_build", format!("{error:?}"))
            })?
            .workspace(workspace_id)
            .map_err(|error| {
                worth_server::WorthServerQueryWorkspaceBindingError::new("workspace_bind", format!("{error:?}"))
            })
        })
    }
}

#[derive(Clone)]
pub struct StatefulCountingMutationRuntimeBackend {
    pub support_profile: WorthQueryRuntimeSupportProfile,
    pub attempted_writes: Arc<AtomicUsize>,
    pub snapshot_version: Arc<AtomicUsize>,
    pub product_source: worth_query::facade::runtime::WorthQueryRelationalSourceOwner,
}

impl StatefulCountingMutationRuntimeBackend {
    pub fn new(
        support_profile: WorthQueryRuntimeSupportProfile,
        attempted_writes: Arc<AtomicUsize>,
        snapshot_version: Arc<AtomicUsize>,
        product_source: worth_query::facade::runtime::WorthQueryRelationalSourceOwner,
    ) -> Self {
        Self {
            support_profile,
            attempted_writes,
            snapshot_version,
            product_source,
        }
    }

    pub fn next_snapshot_ordinal(&self) -> usize {
        self.snapshot_version.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    pub fn current_snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        worth_query::facade::foundation::WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(
            worth_runtime_bridge::facade::TruthSnapshotIdentity::from_relational_snapshot(
                worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts::new(
                    self.snapshot_version.load(std::sync::atomic::Ordering::Relaxed) as u64,
                    1,
                ),
            ),
        )
        .expect("relational snapshot projection must retain its typed payload")
    }
}