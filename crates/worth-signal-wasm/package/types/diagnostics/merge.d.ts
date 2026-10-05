export interface MergePolicyPreviewRequest {
  source_branch_id: number;
  target_branch_id: number;
  conflict_policy_name?: string | null;
  conflict_isolation_policy_name?: string | null;
  identity_matcher_name?: string | null;
  deletion_policy_name?: string | null;
}

export interface MergeSemanticsSummary {
  strategy_name: string;
  strategy_digest: string;
  strategy_basis: string;
  merge_base_name: string;
  merge_base_digest: string;
  merge_base_basis: string;
  conflict_policy_name: string;
  conflict_policy_digest: string;
  conflict_policy_basis: string;
  conflict_isolation_name: string;
  conflict_isolation_digest: string;
  conflict_isolation_basis: string;
  identity_matcher_name: string;
  identity_matcher_digest: string;
  identity_matcher_basis: string;
  source_only_policy_name: string;
  source_only_policy_digest: string;
  source_only_policy_basis: string;
  deletion_policy_name: string;
  deletion_policy_digest: string;
  deletion_policy_basis: string;
}

export interface MergeBaseSummary {
  source_branch_id: number;
  target_branch_id: number;
  forked_from_snapshot_id: number | null;
  source_snapshot_id: number | null;
  target_snapshot_id_before: number | null;
}

export interface LoweredMergeBaseSummary {
  resolved_base: MergeBaseSummary;
  selected_merge_base_name: string;
  selected_merge_base_digest: string;
  selected_merge_base_basis: string;
}

export interface ConflictResolutionRecordSummary {
  source_node: string;
  target_node: string;
  required_resolution: ReadonlyArray<string>;
  supported_strategies: ReadonlyArray<string>;
}

export interface ConflictResolutionPlanSummary {
  source_branch_id: number;
  target_branch_id: number;
  divergence: string;
  records: ReadonlyArray<ConflictResolutionRecordSummary>;
}

export interface MergeNodeMapEntrySummary {
  source_node: string;
  target_node: string;
}

export interface MergeDependencyFingerprintSummary {
  dependency_count: number;
  meaningful_input_changes: number;
  output_hash: string;
}

export interface MergeArtifactAuthoritySummary {
  authority_class: string;
  adoptability: string;
}

export interface MergeComparableSummary {
  output_identity: string | null;
  continuity_token: string | null;
  dependency_fingerprint: MergeDependencyFingerprintSummary;
  authority: MergeArtifactAuthoritySummary;
}

export interface NodeMergeInputStateSummary {
  current_artifact_id: number | null;
  comparable: MergeComparableSummary | null;
  authority: MergeArtifactAuthoritySummary | null;
  exists_in_branch: boolean;
}

export interface NodeMergePlanSummary {
  source_node: string;
  shape_kind: string;
  target_node: string | null;
  source_state: NodeMergeInputStateSummary;
  target_state: NodeMergeInputStateSummary;
  decision: string;
  resolved_conflict_kinds: ReadonlyArray<string>;
}

export interface AdoptionTargetIdentitySummary {
  kind: string;
  mapped_target_node: string | null;
}

export interface AdoptedNodeContractSummary {
  merge_strategy_name: string | null;
  conflict_policy_name: string | null;
  identity_matcher_name: string | null;
  source_only_policy_name: string | null;
  deletion_policy_name: string | null;
  conflict_isolation_policy_name: string | null;
  aspect_merge_policy_binding_count: number;
  condition: string;
  comparator: string | null;
  partitioned_output: boolean;
}

export interface AdoptionPlanCoreSummary {
  source_node: string;
  target_identity: AdoptionTargetIdentitySummary;
  authority: MergeArtifactAuthoritySummary;
  entry_contract: AdoptedNodeContractSummary;
  dependency_count: number;
  dependency_snapshot_edge_count: number;
}

export interface AdoptionCarryPolicySummary {
  runtime_artifact: string;
  retained_artifact: string;
  causality: string;
}

export interface MergePlanArtifact {
  source_branch_id: number;
  target_branch_id: number;
  schema_registry_digest: string;
  registry_bundle_digest: string;
  lowered_strategy_bundle_digest: string;
  merge_kind: string;
  selected_semantics: MergeSemanticsSummary;
  source_snapshot_id: number | null;
  target_snapshot_id_before: number | null;
  merge_base: MergeBaseSummary | null;
  lowered_merge_base: LoweredMergeBaseSummary | null;
  resolution_plan: ConflictResolutionPlanSummary | null;
  node_map: ReadonlyArray<MergeNodeMapEntrySummary>;
  node_plan: ReadonlyArray<NodeMergePlanSummary>;
  adoption_core: ReadonlyArray<AdoptionPlanCoreSummary>;
  adoption_policy: ReadonlyArray<AdoptionCarryPolicySummary>;
}

export interface MergeArtifactRecord {
  source_node: string;
  target_node: string | null;
  source_artifact_id: number | null;
  target_artifact_id_before: number | null;
  target_artifact_id_after: number | null;
  action: string;
  basis: string;
  source_comparable: MergeComparableSummary | null;
  target_comparable: MergeComparableSummary | null;
  identity_basis: string | null;
  identity_status: string | null;
  identity_candidate_count: number;
  resolved_conflict_kinds: ReadonlyArray<string>;
}

export interface MergeCountersSummary {
  boundary_witness_kind: string;
  source_slice_breadth: number;
  proof_minimal_overlap_breadth: number;
  conservative_overlap_expansion_breadth: number;
  final_candidate_breadth: number;
  reconciliation_breadth: number;
  candidate_node_count: number;
  examined_node_count: number;
  adopted_count: number;
  introduced_node_count: number;
  replaced_count: number;
  preserved_target_count: number;
  skipped_non_adoptable_count: number;
  equivalent_unchanged_count: number;
  source_only_count: number;
  target_only_count: number;
  dependency_remap_count: number;
  identity_target_candidates_indexed: number;
  identity_source_lookups: number;
  identity_ambiguous_match_count: number;
  identity_rejected_admissibility_count: number;
  conflict_isolation_record_count: number;
  conflict_isolation_expansion_breadth: number;
  subscriber_repair_breadth: number;
  merge_lineage_record_count: number;
  replay_event_count: number;
}

export interface MergeResultArtifact {
  source_branch: number;
  target_branch: number;
  schema_registry_digest: string;
  registry_bundle_digest: string;
  lowered_strategy_bundle_digest: string;
  merge_kind: string;
  selected_semantics: MergeSemanticsSummary;
  merged_snapshot_id: number | null;
  source_snapshot_id: number | null;
  target_snapshot_id_before: number | null;
  target_snapshot_id_after: number | null;
  lowered_merge_base: LoweredMergeBaseSummary | null;
  resolution_plan: ConflictResolutionPlanSummary | null;
  records: ReadonlyArray<MergeArtifactRecord>;
  counters: MergeCountersSummary;
}

export interface MergePlanProofReport {
  proofSchemaVersion: string;
  registryBundleDigest: string;
  planDigest: string;
  semanticsDigest: string;
  loweredStrategyBundleDigest: string;
  selectedStrategyDigest: string;
  selectedMergeBaseDigest: string;
  selectedConflictPolicyDigest: string;
  selectedConflictIsolationDigest: string;
  selectedIdentityMatcherDigest: string;
  selectedSourceOnlyPolicyDigest: string;
  selectedDeletionPolicyDigest: string;
}

export interface MergeResultProofReport {
  proofSchemaVersion: string;
  registryBundleDigest: string;
  resultDigest: string;
  semanticsDigest: string;
  loweredStrategyBundleDigest: string;
  lineageDigest: string;
  selectedStrategyDigest: string;
  selectedMergeBaseDigest: string;
  selectedConflictPolicyDigest: string;
  selectedConflictIsolationDigest: string;
  selectedIdentityMatcherDigest: string;
  selectedSourceOnlyPolicyDigest: string;
  selectedDeletionPolicyDigest: string;
}

export interface MergePlanProofEnvelope {
  plan: MergePlanArtifact;
  proof: MergePlanProofReport;
}

export interface MergeResultProofEnvelope {
  result: MergeResultArtifact;
  proof: MergeResultProofReport;
}
