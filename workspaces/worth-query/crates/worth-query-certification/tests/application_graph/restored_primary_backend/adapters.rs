//! The smallest bridge-backed backend a Query runtime composes over a
//! restored primary graph. Its write authority commits one empty transaction
//! on main, records the receipt, and then declines, so the runtime's answer
//! is the authority's own refusal whenever index maintenance succeeds.

use std::sync::{Arc, Mutex};

use worth_foundational::facade::{AspectContract, FieldKey};
use worth_query::facade::{domain, foundation, runtime};
use worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeDeliveryReceipt,
    BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode, InvalidationSink,
    MappingSelector, RuntimeBridge, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SliceWideningPolicy, SnapshotReadContract, SubscriptionSliceKind,
    TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
};

/// The refusal the write authority returns after its commit.
pub const DECLINED_AFTER_COMMIT: &str = "the certification write committed, then declined";

pub type CommitLog = Arc<Mutex<Vec<RelationalCommitReceipt>>>;

/// A Bridge over the graph that maps the one field the certification writes.
pub fn runtime_bridge(
    graph: &WorthQueryPrimaryGraphIntegrationHandle,
    contract: &AspectContract,
    field: FieldKey,
) -> RuntimeBridge {
    let identity = "restored-primary-backend";
    let mapping = BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name(identity),
        TruthPatchScope::new(
            MappingSelector::any(),
            AspectKeySelector::exact(contract.key().clone()),
            TruthPatchTargetSelector::entity_field(field),
        ),
        SnapshotReadContract::new(contract.clone()),
        SignalInvalidationScope::from_stable_name(identity),
        CoarseRoutingMode::Direct,
    );
    let aspect = BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::from_stable_name(identity),
        mapping.truth_scope().clone(),
        mapping.snapshot_read_contract().clone(),
        TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    );
    RuntimeBridgeBuilder::new()
        .with_relational_source(graph.relational_bridge_source())
        .with_signal_sink(Sink)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect)
        .build()
        .expect("the certification Bridge over the primary graph builds")
}

struct Sink;

impl InvalidationSink for Sink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
        _lease: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub struct EmptyProjection;

impl runtime::WorthQueryPrimaryGraphSourceProjection for EmptyProjection {
    fn project_live_target(
        &self,
        _graph: &WorthQueryPrimaryGraphIntegrationHandle,
        _target: &runtime::WorthQueryLiveArtifactTarget,
    ) -> Vec<foundation::WorthQueryEntity> {
        Vec::new()
    }

    fn project_granular_scope(
        &self,
        _graph: &WorthQueryPrimaryGraphIntegrationHandle,
        _target: &runtime::WorthQueryLiveArtifactTarget,
        _scope: &domain::WorthQueryMaintenanceScope,
        _basis: &runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<Vec<foundation::WorthQueryEntity>, foundation::WorthQueryWorkspaceError> {
        Ok(Vec::new())
    }
}

pub struct CommitThenDecline(pub CommitLog);

impl runtime::WorthQueryRuntimeWriteAuthorityAdapter for CommitThenDecline {
    fn write(
        &mut self,
        _bridge: &RuntimeBridge,
        relational_runtime: Option<&mut RelationalRuntime>,
        _mutation: runtime::WorthQueryBackendAdmissibleMutation,
    ) -> Result<runtime::WriteAuthorityExecutionReceipt, foundation::WorthQueryWorkspaceError> {
        let runtime = relational_runtime.expect("the primary graph owns the write");
        let main = runtime.main_branch_identity();
        let basis = runtime
            .admit_branch_basis(&main)
            .expect("main is current for the certification write");
        let committed = runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("the transaction binds to main")
            .commit(runtime)
            .expect("the empty certification transaction commits");
        runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .expect("the commit snapshot closes once");
        self.0.lock().unwrap().push(committed.commit.clone());
        Err(foundation::WorthQueryWorkspaceError::new(
            DECLINED_AFTER_COMMIT,
        ))
    }
}

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

pub struct SnapshotAdapter(pub WorthQueryPrimaryGraphIntegrationHandle);

impl runtime::WorthQueryRuntimeSnapshotIdentityAdapter for SnapshotAdapter {
    fn current_snapshot_identity(&self) -> foundation::WorthQuerySnapshotIdentity {
        let branch =
            worth_runtime_bridge::facade::TruthBranchIdentity::from_relational_branch_id("main");
        let snapshot = self
            .0
            .current_truth_snapshot(&branch)
            .expect("the restored primary graph retains its main Bridge head");
        foundation::WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(snapshot)
            .expect("the primary Bridge head is a valid Query snapshot")
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
        runtime::runtime_subscription_support_evidence_identity("restored-primary-backend")
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
                "restored-primary-backend",
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
            "restored-primary-backend",
            receipt.authority_lane(),
            ["restored-primary-backend"],
        ))
    }
}
