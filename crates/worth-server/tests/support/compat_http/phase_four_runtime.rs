#![allow(dead_code)]

use serde_json::Value;
use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey};
use worth_query::facade::foundation::{
    DeclarativeLiveQueryRequest, WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle,
    WorthQueryMutationReceipt, WorthQuerySnapshotIdentity, WorthQueryWorkspaceError,
};
use worth_query::facade::runtime::{
    LiveViewDeclarationAdmissionBoundaryReceipt, QuerySchemaView, SubscriptionActivationInput,
    SubscriptionActivationReceipt, WorthQueryBackendAdmissibleMutation, WorthQueryEvidenceIdentity,
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
    WorthServerQueryWorkspaceBindingTarget, WorthServerQueryWorkspaceProvider,
};

#[path = "phase_four_runtime/request_support.rs"]
pub(crate) mod request_support;

#[derive(Clone, Debug)]
pub(crate) struct StreamingDatasetWorkspaceProvider {
    row_count: usize,
    payload_width: usize,
}

impl StreamingDatasetWorkspaceProvider {
    pub(crate) fn new(row_count: usize, payload_width: usize) -> Self {
        Self {
            row_count,
            payload_width,
        }
    }
}

impl WorthServerQueryWorkspaceProvider for StreamingDatasetWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "streaming-dataset-workspace-provider"
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
        let mut workspace = WorthQueryRuntime::builder(
            worth_query::facade::consumer_kit::in_memory_test_product_world_resources(),
        )
        .backend(StreamingDatasetRuntimeBackend::new(
            self.row_count,
            self.payload_width,
        ))
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
}

#[derive(Clone, Debug)]
struct StreamingDatasetRuntimeBackend {
    row_count: usize,
    payload_width: usize,
}

impl StreamingDatasetRuntimeBackend {
    fn new(row_count: usize, payload_width: usize) -> Self {
        Self {
            row_count,
            payload_width,
        }
    }
}

impl worth_query::facade::runtime::WorthQuerySettlementRecoveryBackend
    for StreamingDatasetRuntimeBackend
{
}

impl worth_query::facade::runtime::WorthQueryMergeSnapshotOwner for StreamingDatasetRuntimeBackend {}

impl WorthQueryRuntimeBackend for StreamingDatasetRuntimeBackend {
    fn support_profile(&self) -> WorthQueryRuntimeSupportProfile {
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile()
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
        Ok(WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, _name: &str) -> Result<(), WorthQueryWorkspaceError> {
        Ok(())
    }

    fn write(
        &mut self,
        _command: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        panic!("phase four streaming runtime does not write")
    }

    fn write_batch(
        &mut self,
        _commands: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<Vec<WorthQueryMutationReceipt>, WorthQueryWorkspaceError> {
        panic!("phase four streaming runtime does not write batches")
    }

    fn execute_intent(
        &mut self,
        _declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryRuntimeError> {
        panic!("phase four streaming runtime does not execute generic intents")
    }

    fn live_entities_for_target(
        &self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryEntity> {
        (0..self.row_count)
            .map(|index| {
                let payload = "x".repeat(self.payload_width);
                WorthQueryEntity::from_native_field_values(
                    worth_query::facade::foundation::WorthQueryEntityIdentity::admit_authored_entity_token(
                        worth_query::facade::foundation::QueryExternalIdentityToken::new(
                            std::sync::Arc::from(format!("stream-row-{index}")),
                        ),
                    ),
                    std::collections::BTreeMap::from([
                        (
                            field_path("identity.id"),
                            AspectValue::String(format!("stream-row-{index}").into()),
                        ),
                        (
                            field_path("payload.value"),
                            AspectValue::String(payload.into()),
                        ),
                    ]),
                )
            })
            .collect()
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
        panic!("phase four streaming runtime does not admit preview basis")
    }

    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError> {
        Ok(WorthQueryRuntimeInspectionEvidence::new(
            authority,
            "phase-four-streaming-inspection",
            receipt.authority_lane(),
            ["phase-four-streaming-runtime"],
        ))
    }
}

fn field_path(path: &str) -> CanonicalFieldPath {
    let fields = path
        .split('.')
        .map(|field| {
            FieldKey::new(field).expect("streaming runtime field segments should be foundational")
        })
        .collect::<Vec<_>>();
    CanonicalFieldPath::new(fields).expect("streaming runtime field path should be non-empty")
}

fn install_requested_named_read(
    workspace: &mut WorthQueryWorkspace,
    request: &WorthServerQueryWorkspaceBindingRequest,
) -> Result<(), WorthServerQueryWorkspaceBindingError> {
    let WorthServerQueryWorkspaceBindingTarget::DirectDeclaration { binding_label, .. } =
        request.target()
    else {
        return Ok(());
    };
    workspace
        .live_view::<Value>(binding_label, |q| {
            q.from("User")
                .select([
                    aspect_field_key("identity", "id"),
                    aspect_field_key("profile", "display_name"),
                ])
                .schema_basis("worth-server-phase-four-streaming")
        })
        .map(|_| ())
        .map_err(|error| {
            WorthServerQueryWorkspaceBindingError::new(
                "workspace_declaration",
                format!("{error:?}"),
            )
        })
}

fn aspect_field_key(aspect: &str, field: &str) -> worth_query::facade::foundation::AspectFieldKey {
    worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(aspect, field)
        .expect("streaming runtime field keys should be foundational")
}

fn streaming_row(index: usize, payload: &str) -> Value {
    serde_json::json!({
        "identity": { "id": format!("user-{index}") },
        "profile": {
            "display_name": format!("Stream User {index}"),
            "payload": payload,
        }
    })
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

struct TestSubscriptionActivation;

impl WorthQueryRuntimeSubscriptionActivationAdapter for TestSubscriptionActivation {
    fn support_evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        worth_query::facade::runtime::runtime_subscription_support_evidence_identity(
            "phase-four-streaming-support",
        )
    }

    fn admit_activation(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<
        worth_query::facade::runtime::SubscriptionActivationBoundaryReceipt,
        WorthQueryWorkspaceError,
    > {
        let receipt = self.build_subscription_activation_receipt(view_name, activation);
        Ok(self.build_subscription_activation_boundary_receipt(view_name, activation, receipt))
    }
}
