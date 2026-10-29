pub use {worth_foundational::facade::AspectValue, worth_foundational::facade::CanonicalFieldPath, worth_foundational::facade::FieldKey};
pub use {worth_query::facade::foundation::DeclarativeLiveQueryRequest, worth_query::facade::foundation::WorthQueryEntity, worth_query::facade::foundation::WorthQueryLivePatch, worth_query::facade::foundation::WorthQueryLiveViewHandle, worth_query::facade::foundation::WorthQuerySnapshotIdentity, worth_query::facade::foundation::WorthQueryWorkspaceError};
pub use {worth_query::facade::runtime::LiveViewDeclarationAdmissionBoundaryReceipt, worth_query::facade::runtime::QuerySchemaView, worth_query::facade::runtime::SubscriptionActivationInput, worth_query::facade::runtime::SubscriptionActivationReceipt, worth_query::facade::runtime::WorthQueryBackendAdmissibleMutation, worth_query::facade::runtime::WorthQueryEvidenceIdentity, worth_query::facade::runtime::WorthQueryLiveArtifactTarget, worth_query::facade::runtime::WorthQueryRuntime, worth_query::facade::runtime::WorthQueryRuntimeBackend, worth_query::facade::runtime::WorthQueryRuntimeError, worth_query::facade::runtime::WorthQueryRuntimeEvidenceAuthority, worth_query::facade::runtime::WorthQueryRuntimeInspectionEvidence, worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection, worth_query::facade::runtime::WorthQueryRuntimeSchemaAdapter, worth_query::facade::runtime::WorthQueryRuntimeSubscriptionActivationAdapter, worth_query::facade::runtime::WorthQueryRuntimeSupportProfile, worth_query::facade::runtime::WorthQuerySessionLabel, worth_query::facade::runtime::WorthQueryWorkspace, worth_query::facade::runtime::WorthQueryWriteReceipt};
pub use {worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts};
pub use {worth_server::WorthServerDirectDeclarationSourceKind, worth_server::WorthServerQueryWorkspaceBindingError, worth_server::WorthServerQueryWorkspaceBindingRequest, worth_server::WorthServerQueryWorkspaceBindingTarget, worth_server::WorthServerQueryWorkspaceProvider};

#[derive(Clone, Debug)]
pub struct RemaskWorkspaceProvider {
    pub projection: WorthQueryRuntimeRemaskProjection,
    pub support_profile: WorthQueryRuntimeSupportProfile,
}

impl RemaskWorkspaceProvider {
    pub fn new(projection: WorthQueryRuntimeRemaskProjection) -> Self {
        Self {
            projection,
            support_profile: worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
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
            let mut workspace = worth_query::facade::runtime::WorthQueryRuntime::builder(resources)
            .backend(crate::shared_support_direct_context_runtime::RemaskRuntimeBackend::new(
                self.support_profile.clone(),
                self.projection.clone(),
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
            })?;
            install_requested_named_read(request_execution, &mut workspace, request)?;
            Ok(workspace)
        })
    }
}

#[derive(Clone)]
pub struct RemaskRuntimeBackend {
    pub support_profile: WorthQueryRuntimeSupportProfile,
    pub projection: WorthQueryRuntimeRemaskProjection,
    pub declared_live_views: std::collections::BTreeSet<String>,
    pub product_source: worth_query::facade::runtime::WorthQueryRelationalSourceOwner,
}

impl RemaskRuntimeBackend {
    pub fn new(
        support_profile: WorthQueryRuntimeSupportProfile,
        projection: WorthQueryRuntimeRemaskProjection,
        product_source: worth_query::facade::runtime::WorthQueryRelationalSourceOwner,
    ) -> Self {
        Self {
            support_profile,
            projection,
            declared_live_views: std::collections::BTreeSet::new(),
            product_source,
        }
    }
}

impl worth_query::facade::runtime::WorthQuerySettlementRecoveryBackend for RemaskRuntimeBackend {}

impl worth_query::facade::runtime::WorthQueryMergeSnapshotOwner for RemaskRuntimeBackend {}

impl WorthQueryRuntimeBackend for RemaskRuntimeBackend {
    fn prepare_product_source(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryProductRelationalInstallation,
        worth_query::facade::runtime::WorthQueryProductSourceDenial,
    > {
        let branch = self
            .product_source
            .with_runtime(request_execution, |runtime| runtime.main_branch_identity());
        self.product_source
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
        DirectContextSchemaAdapter.admit_live_view(name, request, schema_view)
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
        _command: WorthQueryBackendAdmissibleMutation,
    ) -> Result<worth_query::facade::foundation::WorthQueryMutationReceipt, WorthQueryWorkspaceError>
    {
        panic!("unused in direct context remask tests")
    }

    fn write_batch(
        &mut self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        _commands: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<
        Vec<worth_query::facade::foundation::WorthQueryMutationReceipt>,
        WorthQueryWorkspaceError,
    > {
        panic!("unused in direct context remask tests")
    }

    fn execute_intent(
        &mut self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        _declaration: &worth_query::facade::runtime::WorthQueryIntentDeclaration,
    ) -> Result<worth_query::facade::runtime::WorthQueryIntentExecution, WorthQueryRuntimeError>
    {
        panic!("unused in direct context remask tests")
    }

    fn live_entities_for_target(
        &self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryEntity> {
        if self
            .declared_live_views
            .contains(target.terminal_view_name_projection())
        {
            vec![worth_query::facade::foundation::WorthQueryEntity::from_native_field_values(
                    worth_query::facade::foundation::WorthQueryEntityIdentity::admit_authored_entity_token(
                        worth_query::facade::foundation::QueryExternalIdentityToken::new(
                            std::sync::Arc::from("user-1"),
                        ),
                    ),
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
        Ok(worth_query::facade::runtime::WorthQueryRuntimeInspectionEvidence::new(
            authority,
            "direct-context-remask-write-receipt",
            receipt.authority_lane(),
            ["direct-context-remask-inspector"],
        ))
    }
}

pub fn install_requested_named_read(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    request: &WorthServerQueryWorkspaceBindingRequest,
) -> Result<(), WorthServerQueryWorkspaceBindingError> {
    let worth_server::WorthServerQueryWorkspaceBindingTarget::DirectDeclaration {
        source_kind: worth_server::WorthServerDirectDeclarationSourceKind::NamedRead,
        binding_label,
    } = request.target()
    else {
        return Ok(());
    };

    workspace
        .live_view::<serde_json::Value>(request_execution, binding_label, |q| {
            q.from("User")
                .select([
                    aspect_field_key("identity", "id"),
                    aspect_field_key("profile", "display_name"),
                ])
                .schema_basis("worth-server-direct-context-remask")
        })
        .map(|_| ())
        .map_err(|error| {
            worth_server::WorthServerQueryWorkspaceBindingError::new(
                "workspace_declaration",
                format!("{error:?}"),
            )
        })
}

pub fn aspect_field_key(aspect: &str, field: &str) -> worth_query::facade::foundation::AspectFieldKey {
    worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(aspect, field)
        .expect("direct context field keys should be foundational")
}

pub fn field_path(path: &str) -> CanonicalFieldPath {
    let fields = path
        .split('.')
        .map(|field| {
            worth_foundational::facade::FieldKey::new(field).expect("direct context field segments should be foundational")
        })
        .collect::<Vec<_>>();
    worth_foundational::facade::CanonicalFieldPath::new(fields).expect("direct context field path should be non-empty")
}

pub struct DirectContextSchemaAdapter;

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

pub struct DirectContextRemaskActivation {
    pub projection: WorthQueryRuntimeRemaskProjection,
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