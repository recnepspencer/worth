use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey};
use worth_query::facade::foundation::{
    DeclarativeLiveQueryRequest, WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle,
    WorthQuerySnapshotIdentity, WorthQueryWorkspaceError,
};
use worth_query::facade::runtime::{
    LiveViewDeclarationAdmissionBoundaryReceipt, QuerySchemaView, SubscriptionActivationInput,
    SubscriptionActivationReceipt, WorthQueryBackendAdmissibleMutation, WorthQueryEvidenceIdentity,
    WorthQueryLiveArtifactTarget, WorthQueryRuntime, WorthQueryRuntimeBackend,
    WorthQueryRuntimeError, WorthQueryRuntimeEvidenceAuthority,
    WorthQueryRuntimeInspectionEvidence, WorthQueryRuntimeRemaskProjection,
    WorthQueryRuntimeSchemaAdapter, WorthQueryRuntimeSubscriptionActivationAdapter,
    WorthQueryRuntimeSupportProfile, WorthQuerySessionLabel, WorthQueryWorkspace,
    WorthQueryWriteReceipt,
};
use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;
use worth_server::{
    WorthServerDirectDeclarationSourceKind, WorthServerQueryWorkspaceBindingError,
    WorthServerQueryWorkspaceBindingRequest, WorthServerQueryWorkspaceBindingTarget,
    WorthServerQueryWorkspaceProvider,
};

#[derive(Clone, Debug)]
pub(crate) struct RemaskWorkspaceProvider {
    projection: WorthQueryRuntimeRemaskProjection,
    support_profile: WorthQueryRuntimeSupportProfile,
}

impl RemaskWorkspaceProvider {
    pub(crate) fn new(projection: WorthQueryRuntimeRemaskProjection) -> Self {
        Self {
            projection,
            support_profile: WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        }
    }
}

impl WorthServerQueryWorkspaceProvider for RemaskWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "remask-direct-context-workspace-provider"
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
        .backend(RemaskRuntimeBackend::new(
            self.support_profile.clone(),
            self.projection.clone(),
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
struct RemaskRuntimeBackend {
    support_profile: WorthQueryRuntimeSupportProfile,
    projection: WorthQueryRuntimeRemaskProjection,
    declared_live_views: std::collections::BTreeSet<String>,
}

impl RemaskRuntimeBackend {
    fn new(
        support_profile: WorthQueryRuntimeSupportProfile,
        projection: WorthQueryRuntimeRemaskProjection,
    ) -> Self {
        Self {
            support_profile,
            projection,
            declared_live_views: std::collections::BTreeSet::new(),
        }
    }
}

impl worth_query::facade::runtime::WorthQuerySettlementRecoveryBackend for RemaskRuntimeBackend {}

impl worth_query::facade::runtime::WorthQueryMergeSnapshotOwner for RemaskRuntimeBackend {}

impl WorthQueryRuntimeBackend for RemaskRuntimeBackend {
    fn support_profile(&self) -> WorthQueryRuntimeSupportProfile {
        self.support_profile.clone()
    }

    fn admit_live_view_declaration(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError> {
        DirectContextSchemaAdapter.admit_live_view(name, request, schema_view)
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
        _command: WorthQueryBackendAdmissibleMutation,
    ) -> Result<worth_query::facade::foundation::WorthQueryMutationReceipt, WorthQueryWorkspaceError>
    {
        panic!("unused in direct context remask tests")
    }

    fn write_batch(
        &mut self,
        _commands: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<
        Vec<worth_query::facade::foundation::WorthQueryMutationReceipt>,
        WorthQueryWorkspaceError,
    > {
        panic!("unused in direct context remask tests")
    }

    fn execute_intent(
        &mut self,
        _declaration: &worth_query::facade::runtime::WorthQueryIntentDeclaration,
    ) -> Result<worth_query::facade::runtime::WorthQueryIntentExecution, WorthQueryRuntimeError>
    {
        panic!("unused in direct context remask tests")
    }

    fn live_entities_for_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryEntity> {
        if self
            .declared_live_views
            .contains(target.terminal_view_name_projection())
        {
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
        } else {
            Vec::new()
        }
    }

    fn drain_live_patches_for_target(
        &mut self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        Vec::new()
    }

    fn affected_live_view_targets(
        &self,
        _receipt: &worth_query::facade::foundation::WorthQueryMutationReceipt,
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
        let mut activation_adapter = DirectContextRemaskActivation {
            projection: self.projection.clone(),
        };
        let receipt = activation_adapter.admit_activation(view_name, activation)?;
        Ok(receipt.activation_receipt().clone())
    }

    fn admit_preview_basis(
        &self,
        _label: &WorthQuerySessionLabel,
        _effect_policy: worth_query::facade::runtime::WorthQueryEffectPolicy,
        _authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryPreviewBasisAdmission,
        WorthQueryWorkspaceError,
    > {
        panic!("unused in direct context remask tests")
    }

    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError> {
        Ok(WorthQueryRuntimeInspectionEvidence::new(
            authority,
            "direct-context-remask-write-receipt",
            receipt.authority_lane(),
            ["direct-context-remask-inspector"],
        ))
    }
}

fn install_requested_named_read(
    workspace: &mut WorthQueryWorkspace,
    request: &WorthServerQueryWorkspaceBindingRequest,
) -> Result<(), WorthServerQueryWorkspaceBindingError> {
    let WorthServerQueryWorkspaceBindingTarget::DirectDeclaration {
        source_kind: WorthServerDirectDeclarationSourceKind::NamedRead,
        binding_label,
    } = request.target()
    else {
        return Ok(());
    };

    workspace
        .live_view::<serde_json::Value>(binding_label, |q| {
            q.from("User")
                .select([
                    aspect_field_key("identity", "id"),
                    aspect_field_key("profile", "display_name"),
                ])
                .schema_basis("worth-server-direct-context-remask")
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
        .expect("direct context field keys should be foundational")
}

fn field_path(path: &str) -> CanonicalFieldPath {
    let fields = path
        .split('.')
        .map(|field| {
            FieldKey::new(field).expect("direct context field segments should be foundational")
        })
        .collect::<Vec<_>>();
    CanonicalFieldPath::new(fields).expect("direct context field path should be non-empty")
}

struct DirectContextSchemaAdapter;

impl WorthQueryRuntimeSchemaAdapter for DirectContextSchemaAdapter {
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

struct DirectContextRemaskActivation {
    projection: WorthQueryRuntimeRemaskProjection,
}

impl WorthQueryRuntimeSubscriptionActivationAdapter for DirectContextRemaskActivation {
    fn support_evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        worth_query::facade::runtime::runtime_subscription_support_evidence_identity(
            "direct-context-remask-support",
        )
    }

    fn remask_projection(
        &self,
        _view_name: &str,
        _activation: &SubscriptionActivationInput,
    ) -> Option<WorthQueryRuntimeRemaskProjection> {
        Some(self.projection.clone())
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
