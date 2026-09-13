use std::collections::BTreeMap;

use crate::data::error::SignalError;
use crate::logic::transaction::runtime::{
    ArtifactMergeAction, BranchMergePlan, MergeDecisionBasis, MergeTouchedNodeSet,
    MergedArtifactRecord, NodeReconciliationDecision, NodeReconciliationShape,
};
use crate::state::SignalSnapshotId;
use crate::state::SnapshotArtifactRetentionPolicy;

use super::super::branches::{LatestMergeReference, SnapshotBranchState, SnapshotStatePacket};
use super::artifact_projection::node_merge_projection;
use super::execution_preparation::PreparedMergeExecution;

pub(super) struct ArtifactFinalization<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) target_snapshot_after: Option<crate::state::SignalSnapshotId>,
    pub(super) records: Vec<MergedArtifactRecord>,
    pub(super) touched_set: MergeTouchedNodeSet,
    pub(super) merged_source_nodes: Vec<crate::data::handle::NodeId>,
    pub(super) target_snapshot_packet: SnapshotStatePacket<D, I, T>,
    pub(super) node_map: crate::logic::transaction::runtime::MergeNodeMap,
    pub(super) dependency_remaps: Vec<crate::logic::transaction::runtime::DependencyRemapRecord>,
    pub(super) subscriber_repair_breadth: u64,
}

pub(super) fn finalize_artifacts<D, I, T>(
    prepared: &mut PreparedMergeExecution<D, I, T>,
    request: &crate::logic::transaction::runtime::BranchMergeRequest,
    plan: &BranchMergePlan,
    snapshot_id: SignalSnapshotId,
) -> Result<ArtifactFinalization<D, I, T>, SignalError>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    let installed = prepared.target_state.graph().installed_runtime_policy();
    let request_metadata = installed.requested_policy();
    let artifact_retention =
        SnapshotArtifactRetentionPolicy::from_retention_budget(installed.retention_budget());
    let meta = prepared
        .target_state
        .graph_mut()
        .diagnostics_state_mut()
        .allocate_snapshot_meta_with_reserved_id(snapshot_id, request_metadata, artifact_retention);
    prepared
        .target_state
        .graph_mut()
        .diagnostics_state_mut()
        .stage_branch_head_snapshot_projection(request.target_branch.id, meta.snapshot_id);
    let target_snapshot_after = Some(meta.snapshot_id);
    prepared
        .target_state
        .ancestry_mut()
        .set_latest_merge_reference(Some(LatestMergeReference::new(
            request.source_branch.id,
            plan.source_snapshot_id(),
            plan.target_snapshot_id_before(),
            target_snapshot_after,
            plan.merge_kind(),
            plan.merge_strategy(),
        )));
    prepared
        .target_state
        .mutation_ledger_mut()
        .clear_all(target_snapshot_after);
    prepared.target_state.clear_branch_mutation_nodes();

    let identity_records_by_source = plan
        .identity_correspondence()
        .records
        .iter()
        .map(|record| (record.source_node, record))
        .collect::<BTreeMap<_, _>>();
    let mut records = Vec::new();
    for node_plan in plan.node_plan() {
        let target_node = match node_plan.shape() {
            NodeReconciliationShape::ExistingTargetNode { target_node } => Some(target_node),
            NodeReconciliationShape::SourceOnlyIntroduction => {
                prepared.node_map.resolve(node_plan.source_node())
            }
        };
        let target_projection = target_node
            .map(|node| node_merge_projection(prepared.target_state.graph(), node))
            .transpose()?
            .flatten();
        let action = artifact_action(node_plan);
        let identity_record = identity_records_by_source.get(&node_plan.source_node());
        records.push(MergedArtifactRecord {
            source_node: node_plan.source_node(),
            target_node,
            source_artifact_id: node_plan.source_state().current_artifact_id(),
            target_artifact_id_before: node_plan.target_state().current_artifact_id(),
            target_artifact_id_after: target_projection
                .as_ref()
                .and_then(|projection| projection.current_artifact_id),
            action,
            basis: artifact_basis(action),
            source_comparable: node_plan.source_state().comparable().cloned(),
            target_comparable: target_projection
                .as_ref()
                .map(|projection| projection.comparable.clone()),
            identity_basis: identity_record.and_then(|record| record.basis),
            identity_status: identity_record.map(|record| record.status),
            identity_candidate_count: identity_record
                .map(|record| record.candidate_count)
                .unwrap_or_default(),
            resolved_conflict_kinds: node_plan.resolved_conflict_kinds().to_vec(),
        });
    }
    records.sort_by_key(|record| (record.source_node.index(), record.source_node.generation()));
    let touched_set = MergeTouchedNodeSet {
        nodes: prepared.touched.iter().copied().collect(),
    };
    let merged_source_nodes = records
        .iter()
        .filter(|record| !matches!(record.action, ArtifactMergeAction::SkippedNonAdoptable))
        .map(|record| record.source_node)
        .collect();
    let target_snapshot_packet =
        SnapshotBranchState::from_branch_state(&prepared.target_state).packet(meta.snapshot_id);
    Ok(ArtifactFinalization {
        target_snapshot_after,
        records,
        touched_set,
        merged_source_nodes,
        target_snapshot_packet,
        node_map: prepared.node_map.clone(),
        dependency_remaps: prepared.dependency_remaps.clone(),
        subscriber_repair_breadth: prepared.repaired_sources.len() as u64,
    })
}

fn artifact_action(
    node_plan: &crate::logic::transaction::runtime::NodeMergePlan,
) -> ArtifactMergeAction {
    match node_plan.shape() {
        NodeReconciliationShape::SourceOnlyIntroduction => {
            if matches!(
                node_plan.decision(),
                NodeReconciliationDecision::SkipNonAdoptableSource
            ) {
                ArtifactMergeAction::SkippedNonAdoptable
            } else {
                ArtifactMergeAction::IntroducedIntoTarget
            }
        }
        NodeReconciliationShape::ExistingTargetNode { .. } => match node_plan.decision() {
            NodeReconciliationDecision::MarkEquivalentUnchanged => {
                ArtifactMergeAction::EquivalentUnchanged
            }
            NodeReconciliationDecision::PreserveTarget => ArtifactMergeAction::PreservedTarget,
            NodeReconciliationDecision::AdoptSourceAuthority => ArtifactMergeAction::Adopted,
            NodeReconciliationDecision::ReplaceTargetAuthority => ArtifactMergeAction::Replaced,
            NodeReconciliationDecision::SkipNonAdoptableSource
            | NodeReconciliationDecision::RejectRequiresConflictResolution => {
                ArtifactMergeAction::SkippedNonAdoptable
            }
        },
    }
}

fn artifact_basis(action: ArtifactMergeAction) -> MergeDecisionBasis {
    match action {
        ArtifactMergeAction::EquivalentUnchanged => MergeDecisionBasis::EquivalentArtifacts,
        ArtifactMergeAction::IntroducedIntoTarget => {
            MergeDecisionBasis::SourceNodeIntroducedIntoTarget
        }
        ArtifactMergeAction::Adopted | ArtifactMergeAction::Replaced => {
            MergeDecisionBasis::SourceAuthorityAdopted
        }
        ArtifactMergeAction::PreservedTarget => MergeDecisionBasis::MissingSourceArtifact,
        ArtifactMergeAction::SkippedNonAdoptable => MergeDecisionBasis::TargetPreservedNonAdoptable,
    }
}
