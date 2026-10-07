use worth_signal::facade::adapters::{
    BranchMergePlan, BranchMergeResult, MergeNodeMap, MergedArtifactRecord,
};
use worth_signal::facade::NodeId;

use super::{
    AdoptedNodeContractSummary, AdoptionCarryPolicySummary, AdoptionPlanCoreSummary,
    AdoptionTargetIdentitySummary, ConflictResolutionPlanSummary, ConflictResolutionRecordSummary,
    LoweredMergeBaseSummary, MergeArtifactAuthoritySummary, MergeBaseSummary,
    MergeComparableSummary, MergeCountersSummary, MergeNodeMapEntrySummary,
    MergePlanArtifactSummary, MergeRecordSummary, MergeResultArtifactSummary,
    MergeSemanticsSummary, NodeMergeInputStateSummary, NodeMergePlanSummary,
};

impl From<BranchMergePlan> for MergePlanArtifactSummary {
    fn from(value: BranchMergePlan) -> Self {
        Self {
            source_branch_id: value.source_branch_id().0,
            target_branch_id: value.target_branch_id().0,
            schema_registry_digest: value.schema_registry_digest().to_owned(),
            registry_bundle_digest: value.registry_bundle_digest().to_owned(),
            lowered_strategy_bundle_digest: value.lowered_strategy_bundle_digest().to_owned(),
            merge_kind: format!("{:?}", value.merge_kind()),
            selected_semantics: MergeSemanticsSummary::from(value.selected_semantics()),
            source_snapshot_id: value.source_snapshot_id().map(|id| id.0),
            target_snapshot_id_before: value.target_snapshot_id_before().map(|id| id.0),
            merge_base: value.merge_base().map(MergeBaseSummary::from),
            lowered_merge_base: value
                .lowered_merge_base()
                .map(LoweredMergeBaseSummary::from),
            resolution_plan: value
                .resolution_plan()
                .map(|plan| ConflictResolutionPlanSummary {
                    source_branch_id: plan.source_branch_id.0,
                    target_branch_id: plan.target_branch_id.0,
                    divergence: format!("{:?}", plan.divergence),
                    records: plan
                        .records
                        .iter()
                        .map(|record| ConflictResolutionRecordSummary {
                            source_node: node_id_key(record.source_node),
                            target_node: node_id_key(record.target_node),
                            required_resolution: record
                                .required_resolution
                                .iter()
                                .map(|entry| format!("{entry:?}"))
                                .collect(),
                            supported_strategies: record
                                .supported_strategies
                                .iter()
                                .map(|entry| format!("{entry:?}"))
                                .collect(),
                        })
                        .collect(),
                }),
            node_map: merge_node_map_entries(value.node_map()),
            node_plan: value
                .node_plan()
                .iter()
                .map(|entry| NodeMergePlanSummary {
                    source_node: node_id_key(entry.source_node()),
                    shape_kind: match entry.shape() {
                        worth_signal::facade::adapters::NodeReconciliationShape::ExistingTargetNode {
                            ..
                        } => "ExistingTargetNode".to_owned(),
                        worth_signal::facade::adapters::NodeReconciliationShape::SourceOnlyIntroduction => {
                            "SourceOnlyIntroduction".to_owned()
                        }
                    },
                    target_node: match entry.shape() {
                        worth_signal::facade::adapters::NodeReconciliationShape::ExistingTargetNode {
                            target_node,
                        } => Some(node_id_key(target_node)),
                        worth_signal::facade::adapters::NodeReconciliationShape::SourceOnlyIntroduction => {
                            None
                        }
                    },
                    source_state: NodeMergeInputStateSummary {
                        current_artifact_id: entry.source_state().current_artifact_id().map(|id| id.0),
                        comparable: entry
                            .source_state()
                            .comparable()
                            .map(MergeComparableSummary::from),
                        authority: entry.source_state().authority().map(|authority| {
                            MergeArtifactAuthoritySummary {
                                authority_class: format!("{:?}", authority.authority_class),
                                adoptability: format!("{:?}", authority.adoptability),
                            }
                        }),
                        exists_in_branch: entry.source_state().exists_in_branch(),
                    },
                    target_state: NodeMergeInputStateSummary {
                        current_artifact_id: entry.target_state().current_artifact_id().map(|id| id.0),
                        comparable: entry
                            .target_state()
                            .comparable()
                            .map(MergeComparableSummary::from),
                        authority: entry.target_state().authority().map(|authority| {
                            MergeArtifactAuthoritySummary {
                                authority_class: format!("{:?}", authority.authority_class),
                                adoptability: format!("{:?}", authority.adoptability),
                            }
                        }),
                        exists_in_branch: entry.target_state().exists_in_branch(),
                    },
                    decision: format!("{:?}", entry.decision()),
                    resolved_conflict_kinds: entry
                        .resolved_conflict_kinds()
                        .iter()
                        .map(|kind| format!("{kind:?}"))
                        .collect(),
                })
                .collect(),
            adoption_core: value
                .adoption_core()
                .iter()
                .map(|entry| AdoptionPlanCoreSummary {
                    source_node: node_id_key(entry.source_node),
                    target_identity: {
                        let target_identity_json =
                            serde_json::to_value(entry.target_identity).unwrap_or_default();
                        let kind = target_identity_json
                            .as_object()
                            .and_then(|object| object.keys().next().cloned())
                            .unwrap_or_else(|| format!("{:?}", entry.target_identity));
                        let mapped_target_node = target_identity_json
                            .get("ExistingMapping")
                            .and_then(|value| value.get("mapped_target_node"))
                            .and_then(|value| value.as_str())
                            .map(ToOwned::to_owned);
                        AdoptionTargetIdentitySummary {
                            kind,
                            mapped_target_node,
                        }
                    },
                    authority: MergeArtifactAuthoritySummary {
                        authority_class: format!("{:?}", entry.authority.authority_class),
                        adoptability: format!("{:?}", entry.authority.adoptability),
                    },
                    entry_contract: AdoptedNodeContractSummary {
                        merge_strategy_name: entry
                            .entry_contract
                            .eval_config
                            .merge_strategy_name
                            .as_ref()
                            .map(|name| name.as_str().to_owned()),
                        conflict_policy_name: entry
                            .entry_contract
                            .eval_config
                            .conflict_policy_name
                            .as_ref()
                            .map(|name| name.as_str().to_owned()),
                        identity_matcher_name: entry
                            .entry_contract
                            .eval_config
                            .identity_matcher_name
                            .as_ref()
                            .map(|name| name.as_str().to_owned()),
                        source_only_policy_name: entry
                            .entry_contract
                            .eval_config
                            .source_only_policy_name
                            .as_ref()
                            .map(|name| name.as_str().to_owned()),
                        deletion_policy_name: entry
                            .entry_contract
                            .eval_config
                            .deletion_policy_name
                            .as_ref()
                            .map(|name| name.as_str().to_owned()),
                        conflict_isolation_policy_name: entry
                            .entry_contract
                            .eval_config
                            .conflict_isolation_policy_name
                            .as_ref()
                            .map(|name| name.as_str().to_owned()),
                        aspect_merge_policy_binding_count: entry
                            .entry_contract
                            .eval_config
                            .aspect_merge_policy_bindings
                            .len(),
                        condition: format!("{:?}", entry.entry_contract.eval_config.condition),
                        comparator: entry
                            .entry_contract
                            .eval_config
                            .comparator
                            .as_ref()
                            .map(|comparator| format!("{comparator:?}")),
                        partitioned_output: entry.entry_contract.eval_config.partitioned_output,
                    },
                    dependency_count: entry.dependency_topology.dependencies.len(),
                    dependency_snapshot_edge_count: entry
                        .dependency_snapshot_ref
                        .snapshot
                        .entries()
                        .len(),
                })
                .collect(),
            adoption_policy: value
                .adoption_policy()
                .iter()
                .map(|policy| AdoptionCarryPolicySummary {
                    runtime_artifact: format!("{:?}", policy.runtime_artifact),
                    retained_artifact: format!("{:?}", policy.retained_artifact),
                    causality: format!("{:?}", policy.causality),
                })
                .collect(),
        }
    }
}

impl From<BranchMergeResult> for MergeResultArtifactSummary {
    fn from(value: BranchMergeResult) -> Self {
        Self {
            source_branch: value.source_branch.0,
            target_branch: value.target_branch.0,
            schema_registry_digest: value.schema_registry_digest,
            registry_bundle_digest: value.registry_bundle_digest,
            lowered_strategy_bundle_digest: value.lowered_strategy_bundle_digest,
            merge_kind: format!("{:?}", value.merge_kind),
            selected_semantics: MergeSemanticsSummary::from(&value.selected_semantics),
            merged_snapshot_id: value.merged_snapshot_id.map(|id| id.0),
            source_snapshot_id: value.source_snapshot_id.map(|id| id.0),
            target_snapshot_id_before: value.target_snapshot_id_before.map(|id| id.0),
            target_snapshot_id_after: value.target_snapshot_id_after.map(|id| id.0),
            lowered_merge_base: value
                .lowered_merge_base
                .as_ref()
                .map(LoweredMergeBaseSummary::from),
            resolution_plan: value.resolution_plan.as_ref().map(|plan| {
                ConflictResolutionPlanSummary {
                    source_branch_id: plan.source_branch_id.0,
                    target_branch_id: plan.target_branch_id.0,
                    divergence: format!("{:?}", plan.divergence),
                    records: plan
                        .records
                        .iter()
                        .map(|record| ConflictResolutionRecordSummary {
                            source_node: node_id_key(record.source_node),
                            target_node: node_id_key(record.target_node),
                            required_resolution: record
                                .required_resolution
                                .iter()
                                .map(|entry| format!("{entry:?}"))
                                .collect(),
                            supported_strategies: record
                                .supported_strategies
                                .iter()
                                .map(|entry| format!("{entry:?}"))
                                .collect(),
                        })
                        .collect(),
                }
            }),
            records: value.records.iter().map(MergeRecordSummary::from).collect(),
            counters: MergeCountersSummary::from(&value.counters),
        }
    }
}

impl From<&MergedArtifactRecord> for MergeRecordSummary {
    fn from(value: &MergedArtifactRecord) -> Self {
        Self {
            source_node: node_id_key(value.source_node),
            target_node: value.target_node.map(node_id_key),
            source_artifact_id: value.source_artifact_id.map(|id| id.0),
            target_artifact_id_before: value.target_artifact_id_before.map(|id| id.0),
            target_artifact_id_after: value.target_artifact_id_after.map(|id| id.0),
            action: format!("{:?}", value.action),
            basis: format!("{:?}", value.basis),
            source_comparable: value
                .source_comparable
                .as_ref()
                .map(MergeComparableSummary::from),
            target_comparable: value
                .target_comparable
                .as_ref()
                .map(MergeComparableSummary::from),
            identity_basis: value.identity_basis.map(|basis| format!("{basis:?}")),
            identity_status: value.identity_status.map(|status| format!("{status:?}")),
            identity_candidate_count: value.identity_candidate_count,
            resolved_conflict_kinds: value
                .resolved_conflict_kinds
                .iter()
                .map(|kind| format!("{kind:?}"))
                .collect(),
        }
    }
}

fn merge_node_map_entries(value: &MergeNodeMap) -> Vec<MergeNodeMapEntrySummary> {
    value
        .source_to_target
        .iter()
        .map(|(source_node, target_node)| MergeNodeMapEntrySummary {
            source_node: node_id_key(*source_node),
            target_node: node_id_key(*target_node),
        })
        .collect()
}

fn node_id_key(node: NodeId) -> String {
    format!("{}:{}", node.index(), node.generation())
}
