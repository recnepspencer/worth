use std::cell::RefCell;
use std::rc::Rc;

use worth_query::facade::foundation::{
    DeclarativeLiveQueryRequest, WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle,
    WorthQueryMutationDelta, WorthQueryMutationKind, WorthQueryMutationReceipt,
    WorthQuerySnapshotIdentity, WorthQueryWorkspaceError,
};
use worth_query::facade::foundation::{WorthQueryCommitIdentity, WorthQueryEntityIdentity};
use worth_query::facade::runtime::WorthQueryBackendAdmissibleMutation;
use worth_query::facade::runtime::{
    runtime_subscription_support_evidence_identity, LiveViewDeclarationAdmissionBoundaryReceipt,
    QuerySchemaView, SignalInvalidationBoundaryReceipt, SubscriptionActivationBoundaryReceipt,
    SubscriptionActivationInput, WorthQueryBasisAdmissionEvidenceRow, WorthQueryEffectPolicy,
    WorthQueryEvidenceIdentity, WorthQueryLiveArtifactTarget, WorthQueryPreviewBasisAdmission,
    WorthQueryRuntimeEvidenceAuthority, WorthQueryRuntimeInspectionEvidence,
    WorthQueryRuntimeInspectorEvidenceAdapter, WorthQueryRuntimePreviewBasisAdapter,
    WorthQueryRuntimeSchemaAdapter, WorthQueryRuntimeSignalSinkAdapter,
    WorthQueryRuntimeSnapshotIdentityAdapter, WorthQueryRuntimeSourceAdapter,
    WorthQueryRuntimeSubscriptionActivationAdapter, WorthQueryRuntimeWriteAuthorityAdapter,
    WorthQuerySessionLabel, WorthQueryWriteReceipt, WriteAuthorityExecutionReceipt,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts, RuntimeBridge,
    TruthCommitIdentity, TruthSnapshotIdentity,
};

use super::external_row::{apply_aspects_to_external_row, external_row_from_aspects};
use super::state::PublicBridgeRuntimeState;

type SharedRuntimeState = Rc<RefCell<PublicBridgeRuntimeState>>;

pub(super) struct PublicSchemaAdapter;

impl WorthQueryRuntimeSchemaAdapter for PublicSchemaAdapter {
    fn admit_live_view(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        _schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError> {
        let receipt = self.build_live_view_declaration_admission_receipt(name, request);
        Ok(self.build_live_view_declaration_boundary_receipt(name, request, receipt))
    }
}

pub(super) struct PublicSourceAdapter {
    state: SharedRuntimeState,
}

impl PublicSourceAdapter {
    pub(super) fn new(state: SharedRuntimeState) -> Self {
        Self { state }
    }
}

impl WorthQueryRuntimeSourceAdapter for PublicSourceAdapter {
    fn declare_live_view(
        &mut self,
        name: String,
        request: DeclarativeLiveQueryRequest,
        _schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        let live_target =
            WorthQueryLiveArtifactTarget::from_source_adapter_declared_view_name(name.clone());
        self.state
            .borrow_mut()
            .live_views
            .insert(live_target, request.target_collection_identity());
        Ok(WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError> {
        let target = WorthQueryLiveArtifactTarget::from_source_adapter_declared_view_name(name);
        self.state.borrow_mut().live_views.remove(&target);
        Ok(())
    }

    fn live_entities_for_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<WorthQueryEntity>, WorthQueryWorkspaceError> {
        Ok({
            let state = self.state.borrow();
            let Some(collection) = state.live_views.get(target) else {
                return Ok(Vec::new());
            };
            let Some(rows) = state.rows_by_collection.get(collection.as_str()) else {
                return Ok(Vec::new());
            };
            rows.iter()
                .map(|(identity, external_row)| {
                    WorthQueryEntity::from_native_field_values(
                        identity.clone(),
                        external_row.clone(),
                    )
                })
                .collect()
        })
    }

    fn drain_live_patches_for_target(
        &mut self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        Vec::new()
    }

    fn affected_live_view_targets(
        &self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        let state = self.state.borrow();
        let mut affected = receipt
            .deltas()
            .iter()
            .flat_map(|delta| {
                state
                    .live_views
                    .iter()
                    .filter(move |(_, collection)| {
                        delta
                            .target_collection_identity()
                            .same_target_collection_as(collection)
                    })
                    .map(|(target, _)| target.clone())
            })
            .collect::<Vec<_>>();
        affected.sort();
        affected.dedup();
        affected
    }
}

pub(super) struct PublicWriteAuthorityAdapter {
    state: SharedRuntimeState,
}

impl PublicWriteAuthorityAdapter {
    pub(super) fn new(state: SharedRuntimeState) -> Self {
        Self { state }
    }
}

impl WorthQueryRuntimeWriteAuthorityAdapter for PublicWriteAuthorityAdapter {
    fn write(
        &mut self,
        _bridge: &RuntimeBridge,
        _relational_runtime: Option<&mut RelationalRuntime>,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WriteAuthorityExecutionReceipt, WorthQueryWorkspaceError> {
        let mut state = self.state.borrow_mut();
        let collection = mutation
            .declared_collection_identity()
            .map(|collection| collection.as_str().to_string())
            .or_else(|| {
                mutation.existing_truth_binding().and_then(|binding| {
                    binding
                        .terminal_target_collection_projection()
                        .map(str::to_string)
                })
            })
            .or_else(|| {
                mutation
                    .declared_entity_identity_ref()
                    .and_then(|identity| state.collection_by_identity.get(identity).cloned())
            })
            .ok_or_else(|| {
                WorthQueryWorkspaceError::new("public bridge write could not resolve collection")
            })?;
        let entity_identity = match mutation.mutation_family() {
            worth_query::facade::runtime::WorthQueryMutationFamily::Insert => {
                state.next_entity_identity += 1;
                public_entity_identity(RelationalBridgeRecordIdentityParts::entity(
                    1,
                    state.next_entity_identity as u64,
                    0,
                ))
            }
            _ => mutation
                .declared_entity_identity_ref()
                .cloned()
                .or_else(|| {
                    mutation
                        .existing_truth_binding()
                        .map(|binding| binding.resolved_target_identity().clone())
                })
                .or_else(|| {
                    mutation.symbolic_target_reference().and_then(|reference| {
                        state.identity_by_symbol.get(reference.symbol()).cloned()
                    })
                })
                .ok_or_else(|| {
                    WorthQueryWorkspaceError::new(
                        "public bridge write could not resolve target entity identity",
                    )
                })?,
        };
        let mutation_kind = apply_command(&mut state, &mutation, &collection, &entity_identity)?;
        state.next_commit_identity += 1;
        state.next_snapshot_token += 1;
        let commit_identity = public_commit_identity(state.next_commit_identity as u64);
        let snapshot_identity = public_snapshot_identity(state.next_snapshot_token as u64);
        let bridge_authority = self.build_bridge_mutation_authority_bundle(
            _bridge,
            &snapshot_identity,
            &mutation,
            &collection,
            &entity_identity,
            mutation_kind.clone(),
        )?;
        let receipt = WorthQueryMutationReceipt::from_bridge_authoritative_parts(
            commit_identity,
            snapshot_identity,
            vec![WorthQueryMutationDelta::from_touched_aspects(
                collection,
                entity_identity,
                mutation_kind,
                mutation.declared_aspect_touches(),
            )],
            bridge_authority,
        );
        Ok(self.build_write_authority_execution_receipt(&mutation, receipt))
    }
}

fn apply_command(
    state: &mut PublicBridgeRuntimeState,
    mutation: &WorthQueryBackendAdmissibleMutation,
    collection: &str,
    entity_identity: &WorthQueryEntityIdentity,
) -> Result<WorthQueryMutationKind, WorthQueryWorkspaceError> {
    let entity_identity_key = entity_identity.clone();
    match mutation.mutation_family() {
        worth_query::facade::runtime::WorthQueryMutationFamily::Insert => {
            let external_row = external_row_from_aspects(mutation.admitted_aspect_values())?;
            state
                .rows_by_collection
                .entry(collection.to_string())
                .or_default()
                .insert(entity_identity_key.clone(), external_row);
            state
                .collection_by_identity
                .insert(entity_identity_key.clone(), collection.to_string());
            if let Some(reference) = mutation.symbolic_target_reference() {
                state
                    .identity_by_symbol
                    .insert(reference.symbol().to_string(), entity_identity.clone());
            }
            Ok(WorthQueryMutationKind::Created)
        }
        worth_query::facade::runtime::WorthQueryMutationFamily::Update
        | worth_query::facade::runtime::WorthQueryMutationFamily::Assertion => {
            let row = state
                .rows_by_collection
                .entry(collection.to_string())
                .or_default()
                .get_mut(&entity_identity_key)
                .ok_or_else(|| {
                    WorthQueryWorkspaceError::new(format!(
                        "public bridge update could not find `{entity_identity_key:?}` in `{collection}`"
                    ))
                })?;
            apply_aspects_to_external_row(row, mutation.admitted_aspect_values())?;
            Ok(WorthQueryMutationKind::Updated)
        }
        worth_query::facade::runtime::WorthQueryMutationFamily::Delete => {
            if let Some(rows) = state.rows_by_collection.get_mut(collection) {
                rows.remove(&entity_identity_key);
            }
            state.collection_by_identity.remove(&entity_identity_key);
            state.identity_by_symbol.retain(|_, resolved_identity| {
                resolved_identity.evidence_identity() != entity_identity.evidence_identity()
            });
            Ok(WorthQueryMutationKind::Deleted)
        }
    }
}

pub(super) struct PublicSnapshotIdentityAdapter {
    state: SharedRuntimeState,
}

impl PublicSnapshotIdentityAdapter {
    pub(super) fn new(state: SharedRuntimeState) -> Self {
        Self { state }
    }
}

impl WorthQueryRuntimeSnapshotIdentityAdapter for PublicSnapshotIdentityAdapter {
    fn current_snapshot_identity(
        &self,
    ) -> Result<
        WorthQuerySnapshotIdentity,
        worth_query_execution::facade::primary_graph::WorthQueryHandleDenial,
    > {
        Ok({
            let state = self.state.borrow();
            match state.current_snapshot_parts {
                Some((snapshot, version)) => public_snapshot_from_parts(
                    RelationalBridgeSnapshotIdentityParts::new(snapshot, version),
                ),
                None => public_snapshot_identity(state.next_snapshot_token as u64),
            }
        })
    }
}

fn public_snapshot_identity(position: u64) -> WorthQuerySnapshotIdentity {
    public_snapshot_from_parts(RelationalBridgeSnapshotIdentityParts::new(1, position))
}

fn public_snapshot_from_parts(
    parts: RelationalBridgeSnapshotIdentityParts,
) -> WorthQuerySnapshotIdentity {
    WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(
        TruthSnapshotIdentity::from_relational_snapshot(parts),
    )
    .expect("public relational snapshot projection retains typed parts")
}

fn public_commit_identity(position: u64) -> WorthQueryCommitIdentity {
    WorthQueryCommitIdentity::from_bridge_commit_projection(
        TruthCommitIdentity::from_relational_commit_id(position),
    )
}

fn public_entity_identity(parts: RelationalBridgeRecordIdentityParts) -> WorthQueryEntityIdentity {
    WorthQueryEntityIdentity::from_bridge_record_projection(parts)
}

pub(super) struct PublicSignalSinkAdapter;

impl WorthQueryRuntimeSignalSinkAdapter for PublicSignalSinkAdapter {
    fn route_write_receipt(
        &mut self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Result<SignalInvalidationBoundaryReceipt, WorthQueryWorkspaceError> {
        let routed = self.build_signal_invalidation_routing_receipt(receipt)?;
        self.build_signal_invalidation_boundary_receipt(receipt, routed)
    }
}

pub(super) struct PublicSubscriptionActivationAdapter;

impl WorthQueryRuntimeSubscriptionActivationAdapter for PublicSubscriptionActivationAdapter {
    fn support_evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        runtime_subscription_support_evidence_identity("public-graph-subscription-activation")
    }

    fn admit_activation(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationBoundaryReceipt, WorthQueryWorkspaceError> {
        let receipt = self.build_subscription_activation_receipt(view_name, activation);
        Ok(self.build_subscription_activation_boundary_receipt(view_name, activation, receipt))
    }
}

pub(super) struct PublicPreviewBasisAdapter;

impl WorthQueryRuntimePreviewBasisAdapter for PublicPreviewBasisAdapter {
    fn admit_preview_basis(
        &self,
        label: &WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError> {
        Ok(WorthQueryPreviewBasisAdmission::new(
            authority,
            label.clone(),
            effect_policy,
            WorthQueryBasisAdmissionEvidenceRow::rows_from_values(["public-graph-preview-basis"]),
        ))
    }
}

pub(super) struct PublicInspectorEvidenceAdapter;

impl WorthQueryRuntimeInspectorEvidenceAdapter for PublicInspectorEvidenceAdapter {
    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError> {
        Ok(WorthQueryRuntimeInspectionEvidence::new(
            authority,
            "public-graph-write-receipt",
            receipt.authority_lane(),
            ["public-graph-inspector-evidence"],
        ))
    }
}
