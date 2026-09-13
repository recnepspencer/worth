use super::super::*;
use super::merge::{capture_merge_authority, execute_merge, validate_merge_authority};
use super::projection_paths::{
    identity_aspect_key, native_external_field_path_for_aspect_field,
    native_external_field_path_for_grouping_aspect,
    native_external_field_path_for_projection_field,
};
use super::verification::{probe_existing_truth, verify_existing_truth_assertion};
use super::writes::{execute_write, external_row_text_at_path};
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use crate::declarative_live::DeclarativeLiveViewShape;
use crate::memory_workspace::{
    WorthQueryLivePatch, WorthQueryLiveViewHandle, WorthQuerySnapshotIdentity,
};
use crate::subscription::SubscriptionActivationInput;

use super::SharedState;

pub(super) struct StatefulBridgeRuntimeBackend {
    pub(super) state: SharedState,
    support_profile: WorthQueryRuntimeSupportProfile,
}

impl StatefulBridgeRuntimeBackend {
    pub(super) fn new(
        state: SharedState,
        support_profile: WorthQueryRuntimeSupportProfile,
    ) -> Self {
        Self {
            state,
            support_profile,
        }
    }
}

impl WorthQueryMergeSnapshotOwner for StatefulBridgeRuntimeBackend {
    fn release_query_merge_snapshot(
        &mut self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        self.state
            .borrow()
            .relational_source
            .with_runtime_mut(|runtime| runtime.snapshots().release_snapshot(snapshot))
            .expect("merge fixture closes its exact published snapshot once");
    }
}

impl WorthQueryRuntimeBackend for StatefulBridgeRuntimeBackend {
    fn support_profile(&self) -> WorthQueryRuntimeSupportProfile {
        self.support_profile.clone()
    }

    fn prepare_product_source(
        &self,
    ) -> Result<
        worth_query_execution::facade::integration::WorthQueryProductRelationalInstallation,
        crate::runtime::WorthQueryProductSourceDenial,
    > {
        let source = self.state.borrow().relational_source.clone();
        let branch = source.with_runtime(|runtime| runtime.main_branch_identity());
        source
            .prepare_product_source(&branch)
            .map_err(crate::runtime::WorthQueryProductSourceDenial::Basis)
    }

    fn current_snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        let state = self.state.borrow();
        WorthQuerySnapshotIdentity::preview(
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::RuntimeStateSnapshot)
                .field_usize(
                    WorthQueryEvidenceTag::new("stateful_bridge_snapshot_sequence"),
                    state.next_snapshot_token,
                )
                .seal(),
        )
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
        request: DeclarativeLiveQueryRequest,
        _schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        let live_target = WorthQueryLiveArtifactTarget::from_view_name(name.clone());
        self.state
            .borrow_mut()
            .live_views
            .insert(live_target, request.target_collection_identity());
        Ok(WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError> {
        self.state
            .borrow_mut()
            .live_views
            .remove(&WorthQueryLiveArtifactTarget::from_view_name(name));
        Ok(())
    }

    fn write(
        &mut self,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        execute_write(&self.state, mutation)
    }

    fn write_batch(
        &mut self,
        mutations: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<Vec<WorthQueryMutationReceipt>, WorthQueryWorkspaceError> {
        let mut receipts = Vec::with_capacity(mutations.len());
        for mutation in mutations {
            receipts.push(self.write(mutation)?);
        }
        Ok(receipts)
    }

    fn admit_existing_truth_binding(
        &self,
        binding: &WorthQueryExistingTruthTargetBinding,
    ) -> Result<(), WorthQueryExistingTruthBindingDenial> {
        let state = self.state.borrow();
        if let Some(expected_collection) = binding.terminal_target_collection_projection() {
            if !state.installed_collections.contains(expected_collection) {
                return Err(WorthQueryExistingTruthBindingDenial::new(
                    binding,
                    WorthQueryExistingTruthBindingDenialKind::CollectionMismatch,
                    format!("declared target collection `{expected_collection}` is not installed"),
                ));
            }
        }
        let resolved_target_identity = binding
            .resolved_target_identity()
            .terminal_projection_for_reporting();
        let Some(actual_collection) = state.collection_by_identity.get(&resolved_target_identity)
        else {
            return Err(WorthQueryExistingTruthBindingDenial::new(
                binding,
                WorthQueryExistingTruthBindingDenialKind::ResolvedTargetMissing,
                format!(
                    "resolved target `{}` is not present in authoritative truth",
                    resolved_target_identity
                ),
            ));
        };
        if let Some(expected_collection) = binding.terminal_target_collection_projection() {
            if actual_collection != expected_collection {
                return Err(WorthQueryExistingTruthBindingDenial::new(
                    binding,
                    WorthQueryExistingTruthBindingDenialKind::CollectionMismatch,
                    format!(
                        "resolved target `{}` belongs to collection `{actual_collection}`, not `{expected_collection}`",
                        resolved_target_identity
                    ),
                ));
            }
        }
        Ok(())
    }

    fn verify_existing_truth_assertion(
        &self,
        binding: &WorthQueryExistingTruthTargetBinding,
        aspects: &[WorthQueryAuthoredAspectMutation],
    ) -> Result<WorthQueryVerifiedExistingTruthAssertion, WorthQueryExistingTruthAssertionDenial>
    {
        let state = self.state.borrow();
        let snapshot_identity = self.current_snapshot_identity();
        verify_existing_truth_assertion(&state, binding, aspects, snapshot_identity)
    }

    fn probe_existing_truth(
        &self,
        request: &WorthQueryExistingTruthProbeRequest,
    ) -> Result<WorthQueryExistingTruthProbe, WorthQueryExistingTruthProbeDenial> {
        probe_existing_truth(&self.state.borrow(), request)
    }

    fn execute_intent(
        &mut self,
        _declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryRuntimeError> {
        Err(WorthQueryRuntimeError::MissingIntentAuthority)
    }

    fn admit_query_writeback_authority(&self) -> Result<(), WorthQueryWorkspaceError> {
        if self.state.borrow().bridge.writeback_authority().is_some() {
            Ok(())
        } else {
            Err(WorthQueryWorkspaceError::new(
                "stateful bridge fixture has no truth writeback authority",
            ))
        }
    }

    fn execute_query_writeback(
        &mut self,
        declaration: &crate::workflow::QueryWritebackDeclaration,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeAdmittedWritebackExecution,
        (crate::effect_lifecycle::EffectExecutionDenialKind, String),
    > {
        crate::effect_lifecycle::execute_lowered_writeback(&self.state.borrow().bridge, declaration)
    }

    fn capture_query_merge_authority(
        &self,
        target_branch: &crate::runtime::WorthQueryAdmittedBranchName,
        source_branch: &crate::runtime::WorthQueryAdmittedBranchName,
    ) -> Result<WorthQueryBackendMergeAuthority, WorthQueryWorkspaceError> {
        capture_merge_authority(&self.state, target_branch, source_branch)
    }

    fn validate_query_merge_authority(
        &self,
        authority: &WorthQueryBackendMergeAuthority,
    ) -> Result<(), WorthQueryWorkspaceError> {
        validate_merge_authority(&self.state, authority)
    }

    fn execute_query_merge(
        &mut self,
        authority: &WorthQueryBackendMergeAuthority,
        declaration: &crate::workflow::LoweredMergeWorkflowDeclaration,
    ) -> Result<
        worth_relational::facade::transactions::MergeExecutionOutcome,
        crate::effect_lifecycle::RelationalEffectExecutionFailure,
    > {
        execute_merge(&self.state, authority, declaration)
    }

    fn execute_query_causal_inspection(
        &self,
        plan: &crate::runtime::CausalInspectionPlan,
    ) -> Result<
        crate::runtime::QueryCausalInspectionArtifact,
        crate::runtime::WorthQueryBackendInspectionError,
    > {
        plan.materialize_with_bridge(&self.state.borrow().bridge)
            .map_err(Into::into)
    }

    fn live_entities_for_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryEntity> {
        let state = self.state.borrow();
        let Some(collection) = state.live_views.get(target) else {
            return Vec::new();
        };
        let Some(rows) = state.rows_by_collection.get(collection.as_str()) else {
            return Vec::new();
        };
        rows.iter()
            .map(|(identity, external_row)| {
                WorthQueryEntity::from_native_field_values(
                    state
                        .identity_by_storage_key
                        .get(identity)
                        .cloned()
                        .unwrap_or_else(|| {
                            crate::memory_workspace::admit_authored_entity_label(identity)
                        }),
                    external_row.clone(),
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
        receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        let state = self.state.borrow();
        let mut affected = receipt
            .deltas
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

    fn install_live_subscription(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationReceipt, WorthQueryWorkspaceError> {
        let receipt = TestSubscriptionActivation.admit_activation(view_name, activation)?;
        Ok(receipt.activation_receipt().clone())
    }

    fn admit_preview_basis(
        &self,
        label: &WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError> {
        TestPreviewBasis.admit_preview_basis(label, effect_policy, authority)
    }

    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError> {
        TestInspectorEvidence.inspect_write_receipt(receipt, authority)
    }

    fn grouped_baseline_members(
        &self,
        request: &DeclarativeLiveQueryRequest,
    ) -> Result<
        Option<Vec<crate::view_shape_live::WorthQueryGroupedBaselineMember>>,
        WorthQueryWorkspaceError,
    > {
        let DeclarativeLiveViewShape::KanbanGrouped { grouping_aspect } = request.view_shape()
        else {
            return Ok(None);
        };
        let identity_path = request
            .projection()
            .iter()
            .find(|field| field.source_field_key().native_aspect_key() == identity_aspect_key())
            .map(native_external_field_path_for_projection_field)
            .unwrap_or_else(|| native_external_field_path_for_aspect_field("identity", "id"));
        let identity_path = identity_path?;
        let grouping_path = request
            .projection()
            .iter()
            .find(|field| field.source_field_key().native_aspect_key() == *grouping_aspect)
            .map(native_external_field_path_for_projection_field)
            .unwrap_or_else(|| native_external_field_path_for_grouping_aspect(grouping_aspect));
        let grouping_path = grouping_path?;
        let members = self
            .state
            .borrow()
            .rows_by_collection
            .get(request.target())
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter_map(|(entity_identity, external_row)| {
                let member = external_row_text_at_path(external_row, &identity_path)
                    .unwrap_or_else(|| entity_identity.clone());
                let lane = external_row_text_at_path(external_row, &grouping_path)?;
                Some(
                    crate::view_shape_live::WorthQueryGroupedBaselineMember::from_authoritative_member_lane_keys(
                        member,
                        lane,
                    ),
                )
            })
            .collect::<Vec<_>>();
        Ok(Some(members))
    }
}
