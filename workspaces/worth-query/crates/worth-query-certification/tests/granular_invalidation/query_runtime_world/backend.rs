use worth_query::facade::{foundation, runtime};

pub struct SchemaAdapter;

impl runtime::WorthQueryRuntimeSchemaAdapter for SchemaAdapter {
    fn admit_live_view(
        &self,
        name: &str,
        request: &foundation::DeclarativeLiveQueryRequest,
        _schema_view: &runtime::QuerySchemaView,
    ) -> Result<
        runtime::LiveViewDeclarationAdmissionBoundaryReceipt,
        foundation::WorthQueryWorkspaceError,
    > {
        let receipt = self.build_live_view_declaration_admission_receipt(name, request);
        Ok(self.build_live_view_declaration_boundary_receipt(name, request, receipt))
    }
}

pub struct PrimarySnapshotAdapter(
    worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
);

impl PrimarySnapshotAdapter {
    pub fn new(
        installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    ) -> Self {
        Self(installation.retain_primary_graph_integration_handle())
    }
}

impl runtime::WorthQueryRuntimeSnapshotIdentityAdapter for PrimarySnapshotAdapter {
    fn current_snapshot_identity(
        &self,
    ) -> Result<foundation::WorthQuerySnapshotIdentity, runtime::WorthQueryHandleDenial> {
        Ok({
            let branch =
                worth_runtime_bridge::facade::TruthBranchIdentity::from_relational_branch_id(
                    "main",
                );
            let snapshot = self
                .0
                .current_truth_snapshot(&branch)?
                .expect("the published fixture retains its exact Bridge head");
            foundation::WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(snapshot)
                .expect("the primary Bridge head is a valid Query snapshot")
        })
    }
}

pub struct DenyingWriteAuthority;

impl runtime::WorthQueryRuntimeWriteAuthorityAdapter for DenyingWriteAuthority {
    fn write(
        &mut self,
        _bridge: &worth_runtime_bridge::facade::RuntimeBridge,
        _relational_runtime: Option<&mut worth_relational::facade::runtime::RelationalRuntime>,
        _mutation: runtime::WorthQueryBackendAdmissibleMutation,
    ) -> Result<runtime::WriteAuthorityExecutionReceipt, foundation::WorthQueryWorkspaceError> {
        Err(foundation::WorthQueryWorkspaceError::new(
            "the certification Query consumer is read-only",
        ))
    }
}

pub struct SignalSink;

impl runtime::WorthQueryRuntimeSignalSinkAdapter for SignalSink {
    fn route_write_receipt(
        &mut self,
        receipt: &foundation::WorthQueryMutationReceipt,
    ) -> Result<runtime::SignalInvalidationBoundaryReceipt, foundation::WorthQueryWorkspaceError>
    {
        let routed = self.build_signal_invalidation_routing_receipt(receipt)?;
        self.build_signal_invalidation_boundary_receipt(receipt, routed)
    }
}

pub struct SubscriptionActivation;

impl runtime::WorthQueryRuntimeSubscriptionActivationAdapter for SubscriptionActivation {
    fn support_evidence_identity(&self) -> runtime::WorthQueryEvidenceIdentity {
        runtime::runtime_subscription_support_evidence_identity(
            "primary-graph-granular-certification",
        )
    }

    fn admit_activation(
        &mut self,
        view_name: &str,
        activation: &runtime::SubscriptionActivationInput,
    ) -> Result<runtime::SubscriptionActivationBoundaryReceipt, foundation::WorthQueryWorkspaceError>
    {
        let receipt = self.build_subscription_activation_receipt(view_name, activation);
        Ok(self.build_subscription_activation_boundary_receipt(view_name, activation, receipt))
    }
}

pub struct PreviewBasis;

impl runtime::WorthQueryRuntimePreviewBasisAdapter for PreviewBasis {
    fn admit_preview_basis(
        &self,
        label: &runtime::WorthQuerySessionLabel,
        effect_policy: runtime::WorthQueryEffectPolicy,
        authority: &runtime::WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<runtime::WorthQueryPreviewBasisAdmission, foundation::WorthQueryWorkspaceError>
    {
        Ok(runtime::WorthQueryPreviewBasisAdmission::new(
            authority,
            label.clone(),
            effect_policy,
            runtime::WorthQueryBasisAdmissionEvidenceRow::rows_from_values([
                "primary-graph-granular-certification",
            ]),
        ))
    }
}

pub struct InspectorEvidence;

impl runtime::WorthQueryRuntimeInspectorEvidenceAdapter for InspectorEvidence {
    fn inspect_write_receipt(
        &self,
        receipt: &runtime::WorthQueryWriteReceipt,
        authority: &runtime::WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<runtime::WorthQueryRuntimeInspectionEvidence, foundation::WorthQueryWorkspaceError>
    {
        Ok(runtime::WorthQueryRuntimeInspectionEvidence::new(
            authority,
            "primary-graph-granular-certification",
            receipt.authority_lane(),
            ["primary-graph-granular-certification"],
        ))
    }
}
