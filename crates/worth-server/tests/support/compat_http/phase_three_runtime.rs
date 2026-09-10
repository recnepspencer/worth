#![allow(dead_code)]

#[path = "phase_three_runtime/backend.rs"]
mod phase_three_backend;

use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_proof::TransitionOutcome;
use worth_query::facade::foundation::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQueryMutationDelta,
    WorthQueryMutationKind,
};
use worth_query::facade::foundation::{
    WorthQueryEntity, WorthQueryLivePatch, WorthQueryMutationReceipt, WorthQuerySnapshotIdentity,
    WorthQueryWorkspaceError,
};
use worth_query::facade::runtime::{
    SubscriptionActivationInput, SubscriptionActivationReceipt,
    WorthQueryBackendAdmissibleMutation, WorthQueryIntentDeclaration, WorthQueryIntentExecution,
    WorthQueryLiveArtifactTarget, WorthQueryPreviewBasisAdmission, WorthQueryRuntime,
    WorthQueryRuntimeBackend, WorthQueryRuntimeError, WorthQueryRuntimeEvidenceAuthority,
    WorthQueryRuntimeInspectionEvidence, WorthQueryRuntimeSupportProfile, WorthQuerySessionLabel,
    WorthQueryWorkspace, WorthQueryWriteCommand, WorthQueryWriteReceipt,
};
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
};
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatHttpRouteFamily, WorthServerCompatibilityMutationExecutionInput,
    WorthServerCompatibilityPreparedRequest, WorthServerCompatibilityRequestInput,
    WorthServerConfig, WorthServerMiddlewareConfig, WorthServerQueryHandoffConfig,
    WorthServerQueryWorkspaceBindingError, WorthServerQueryWorkspaceBindingRequest,
    WorthServerQueryWorkspaceProvider, WorthServerRequestContextConfig,
};

use crate::query_handoff_runtime::runtime_aspect_contracts::query_handoff_aspect_contracts;

pub(crate) fn build_phase_three_server() -> WorthServer {
    build_phase_three_server_with_workspace_provider(
        crate::query_handoff_runtime::TestWorkspaceProvider,
    )
}

pub(crate) fn build_phase_three_server_with_workspace_provider(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
) -> WorthServer {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
}

pub(crate) fn mutation_input(
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(WorthServerCompatHttpRouteFamily::Mutation)
        .with_method("POST")
        .with_path(format!("/compat/mutations/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type("application/json")
        .with_body_present(true)
}

pub(crate) fn prepared_mutation_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility mutation request, got {other:?}"),
    }
}

pub(crate) fn compat_mutation_execution_input(
    server: &WorthServer,
    operation_name: &str,
    body: Value,
) -> WorthServerCompatibilityMutationExecutionInput {
    WorthServerCompatibilityMutationExecutionInput::new(
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

pub(crate) fn single_insert_body(identity: &str) -> Value {
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

pub(crate) fn mutation_request_input_for_workspace(
    operation_name: &str,
    workspace_id: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id(workspace_id)
        .with_branch_id("branch-9")
        .with_route_family(WorthServerCompatHttpRouteFamily::Mutation)
        .with_method("POST")
        .with_path(format!("/compat/mutations/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type("application/json")
        .with_body_present(true)
}

pub(crate) fn insert_task(identity: &str) -> WorthQueryWriteCommand {
    worth_query::facade::runtime::WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", identity)
        .aspect("title.value", format!("Title for {identity}"))
        .build_insert("Task")
        .expect("insert command should build")
}

pub(crate) fn compat_mutation_success(
    outcome: worth_server::WorthServerCompatibilityMutationOutcome<
        worth_server::WorthServerCompatibilityMutation,
    >,
) -> worth_server::WorthServerCompatibilityMutation {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility mutation success, got {other:?}"),
    }
}

pub(crate) fn compat_mutation_denied(
    outcome: worth_server::WorthServerCompatibilityMutationOutcome<
        worth_server::WorthServerCompatibilityMutation,
    >,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility mutation denial, got {other:?}"),
    }
}

pub(crate) fn direct_mutation_success(
    outcome: worth_server::WorthServerDirectMutationOutcome,
) -> worth_server::WorthServerDirectMutation {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected direct mutation success, got {other:?}"),
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StatefulCountingMutationWorkspaceProvider {
    support_profile: WorthQueryRuntimeSupportProfile,
    attempted_writes: Arc<AtomicUsize>,
    snapshot_version: Arc<AtomicUsize>,
}

impl StatefulCountingMutationWorkspaceProvider {
    pub(crate) fn new(
        support_profile: WorthQueryRuntimeSupportProfile,
        attempted_writes: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            support_profile,
            attempted_writes,
            snapshot_version: Arc::new(AtomicUsize::new(0)),
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
        let workspace_id = request
            .resolved_request_context()
            .request_context()
            .workspace_target()
            .workspace_id();
        WorthQueryRuntime::builder(
            worth_query::facade::consumer_kit::in_memory_test_product_world_resources(),
        )
        .aspect_contracts(query_handoff_aspect_contracts())
        .map_err(|error| {
            WorthServerQueryWorkspaceBindingError::new(
                "aspect_contracts",
                format!("failed to install compatibility mutation aspect contracts: {error}"),
            )
        })?
        .backend(StatefulCountingMutationRuntimeBackend::new(
            self.support_profile.clone(),
            self.attempted_writes.clone(),
            self.snapshot_version.clone(),
        ))
        .build()
        .map_err(|error| {
            WorthServerQueryWorkspaceBindingError::new("runtime_build", format!("{error:?}"))
        })?
        .workspace(workspace_id)
        .map_err(|error| {
            WorthServerQueryWorkspaceBindingError::new("workspace_bind", format!("{error:?}"))
        })
    }
}

#[derive(Clone, Debug)]
struct StatefulCountingMutationRuntimeBackend {
    support_profile: WorthQueryRuntimeSupportProfile,
    attempted_writes: Arc<AtomicUsize>,
    snapshot_version: Arc<AtomicUsize>,
}

impl StatefulCountingMutationRuntimeBackend {
    fn new(
        support_profile: WorthQueryRuntimeSupportProfile,
        attempted_writes: Arc<AtomicUsize>,
        snapshot_version: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            support_profile,
            attempted_writes,
            snapshot_version,
        }
    }

    fn next_snapshot_ordinal(&self) -> usize {
        self.snapshot_version.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn current_snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(
            worth_runtime_bridge::facade::TruthSnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(
                    self.snapshot_version.load(Ordering::Relaxed) as u64,
                    1,
                ),
            ),
        )
        .expect("relational snapshot projection must retain its typed payload")
    }
}
