use super::*;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct MockDownstreamEvaluationMetrics {
    pub(super) nodes_evaluated: u64,
    pub(super) nodes_recomputed: u64,
    pub(super) suppressed_downstream_propagations: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct MockDownstreamPlannerMetrics {
    pub(super) tasks_scheduled: u64,
    pub(super) tasks_pruned_before_execution: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct MockDownstreamObservation {
    pub(super) evaluation: MockDownstreamEvaluationMetrics,
    pub(super) planner: MockDownstreamPlannerMetrics,
}

#[derive(Debug)]
pub(super) struct MockDownstreamRuntime {
    pub(super) development_profile: bool,
    pub(super) source_versions: Vec<u64>,
    pub(super) intermediate_versions: Vec<u64>,
    pub(super) target_versions: Vec<u64>,
    pub(super) observation: MockDownstreamObservation,
    pub(super) history_entries: usize,
    pub(super) has_latest_flow: bool,
}

#[derive(Debug)]
pub(super) struct GameEngineFrameSeedOutcome {
    pub(super) entities: Vec<crate::facade::identity::EntityId>,
    pub(super) frame_targets: Vec<crate::facade::identity::EntityId>,
    pub(super) explicit_targets: Vec<crate::facade::identity::EntityId>,
    pub(super) propagation_seeds: Vec<crate::facade::identity::EntityId>,
    pub(super) relation_count: usize,
    pub(super) region_count: usize,
}

impl MockDownstreamRuntime {
    pub(super) fn new(development_profile: bool, source_count: usize) -> Self {
        let bounded = source_count.max(4);
        Self {
            development_profile,
            source_versions: vec![1; bounded],
            intermediate_versions: (0..bounded).map(|index| 100 + index as u64 * 10).collect(),
            target_versions: (0..bounded)
                .map(|index| 1_000 + index as u64 * 100)
                .collect(),
            observation: MockDownstreamObservation::default(),
            history_entries: 0,
            has_latest_flow: false,
        }
    }

    pub(super) fn warmup(&mut self) {
        self.apply_changes(self.source_versions.len());
        self.history_entries = 0;
        self.has_latest_flow = false;
        self.observation = MockDownstreamObservation::default();
    }

    pub(super) fn observe(&self) -> MockDownstreamObservation {
        self.observation
    }

    pub(super) fn recent_history_len(&self) -> usize {
        self.history_entries
    }

    pub(super) fn latest_flow_diagnostics(&self) -> Option<()> {
        self.has_latest_flow.then_some(())
    }

    pub(super) fn apply_changes(&mut self, affected_sources: usize) {
        let bounded = affected_sources.min(self.source_versions.len());
        let mut affected_targets = BTreeSet::new();
        for index in 0..bounded {
            self.source_versions[index] += 1;
            self.intermediate_versions[index] += 10;
            self.target_versions[index] += 100;
            affected_targets.insert(index);
            affected_targets
                .insert((index + self.target_versions.len() - 1) % self.target_versions.len());
        }

        let affected_target_count = affected_targets.len() as u64;
        let bounded = bounded as u64;
        let tasks_scheduled = bounded + affected_target_count;
        let nodes_recomputed = bounded * 2 + affected_target_count;
        let nodes_evaluated = if self.development_profile {
            nodes_recomputed + affected_target_count
        } else {
            nodes_recomputed
        };
        let tasks_pruned_before_execution = if self.development_profile {
            0
        } else {
            bounded / 2
        };
        let suppressed_downstream_propagations = if self.development_profile {
            0
        } else {
            bounded.saturating_sub(1)
        };

        self.observation.evaluation.nodes_evaluated += nodes_evaluated;
        self.observation.evaluation.nodes_recomputed += nodes_recomputed;
        self.observation
            .evaluation
            .suppressed_downstream_propagations += suppressed_downstream_propagations;
        self.observation.planner.tasks_scheduled += tasks_scheduled;
        self.observation.planner.tasks_pruned_before_execution += tasks_pruned_before_execution;
        self.history_entries += if self.development_profile { 3 } else { 1 };
        self.has_latest_flow = true;
    }
}

pub(super) fn diagnostic_artifact_kind_count(
    artifacts: &[crate::facade::diagnostics::RelationalDiagnosticArtifact],
    kind: DiagnosticsArtifactKind,
) -> usize {
    artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind)
        .count()
}

pub(super) fn diagnostic_artifact_scope_count(
    artifacts: &[crate::facade::diagnostics::RelationalDiagnosticArtifact],
    scope: DiagnosticsScope,
) -> usize {
    artifacts
        .iter()
        .filter(|artifact| artifact.scope == scope)
        .count()
}

pub(super) fn diagnostic_entry_code_count(
    artifacts: &[crate::facade::diagnostics::RelationalDiagnosticArtifact],
    code: DiagnosticCode,
) -> usize {
    artifacts
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .filter(|entry| entry.code == code)
        .count()
}

pub(super) fn diagnostic_entry_count(
    artifacts: &[crate::facade::diagnostics::RelationalDiagnosticArtifact],
) -> usize {
    artifacts
        .iter()
        .map(|artifact| artifact.entries.len())
        .sum()
}

pub(super) fn runtime_execution_lane_code(profile: RelationalRuntimeProfile) -> u64 {
    match profile.boundary_policy().execution_lane {
        crate::facade::config::RuntimeExecutionLane::OperationalThin => 1,
        crate::facade::config::RuntimeExecutionLane::RichInteractive => 2,
        crate::facade::config::RuntimeExecutionLane::AuditReplayHeavy => 3,
    }
}

pub(super) fn diagnostics_boundary_code(profile: RelationalRuntimeProfile) -> u64 {
    match profile.boundary_policy().diagnostics_boundary {
        crate::facade::config::DiagnosticsBoundary::MinimalHotTruth => 1,
        crate::facade::config::DiagnosticsBoundary::RichCertification => 2,
        crate::facade::config::DiagnosticsBoundary::DurableWorkflow => 3,
    }
}

pub(super) fn profile_boundary_metrics(
    runtime: &crate::runtime::RelationalRuntime,
    profile: RelationalRuntimeProfile,
) -> PerfMetricSet {
    let boundary = runtime.config.boundary_policy();
    perf_metrics!({
        "execution_lane_code": runtime_execution_lane_code(profile),
        "diagnostics_boundary_code": diagnostics_boundary_code(profile),
        "prefers_checkpoint_compaction": u64::from(boundary.prefers_checkpoint_compaction),
        "allows_compiled_lane": u64::from(boundary.allows_compiled_lane),
        "keeps_replay_hot_path_thin": u64::from(boundary.keeps_replay_hot_path_thin),
        "matches_defaults": u64::from(runtime.config.profile_boundary_matches_defaults()),
    })
}

pub(super) fn build_mock_downstream_runtime(
    development_profile: bool,
    source_count: usize,
) -> MockDownstreamRuntime {
    let mut runtime = MockDownstreamRuntime::new(development_profile, source_count);
    runtime.warmup();
    runtime
}

pub(super) fn seed_downstream_region_world(
    runtime: &RelationalRuntime,
    label: &str,
    node_count: usize,
    cross_link_stride: usize,
) -> Vec<crate::facade::identity::EntityId> {
    let mut entities = Vec::with_capacity(node_count);
    for index in 0..node_count {
        let created = create_entity_outcome(runtime, &format!("{label}-node-{index}"));
        entities.push(changed_entities(&created)[0]);
    }

    for window in entities.windows(2).enumerate() {
        let (index, pair) = window;
        create_relation_outcome(
            runtime,
            pair[0],
            pair[1],
            &format!("{label}-link-chain-{index}"),
        );
    }

    if cross_link_stride > 1 {
        for index in 0..entities.len() {
            let target = index + cross_link_stride;
            if target < entities.len() {
                create_relation_outcome(
                    runtime,
                    entities[index],
                    entities[target],
                    &format!("{label}-link-cross-{index}-{target}"),
                );
            }
        }
    }

    entities
}

pub(super) fn seed_game_engine_frame_world(
    runtime: &RelationalRuntime,
    label: &str,
    region_count: usize,
    nodes_per_region: usize,
) -> GameEngineFrameSeedOutcome {
    let mut entities = Vec::with_capacity(region_count * nodes_per_region);
    let mut frame_targets = Vec::with_capacity(region_count);
    let mut explicit_targets = Vec::with_capacity(region_count * 2);
    let mut propagation_seeds = Vec::with_capacity(region_count.min(6));
    let mut relation_count = 0usize;

    for region in 0..region_count {
        let partition_id = PartitionId(700 + region as u32);
        let region_start = entities.len();
        for node in 0..nodes_per_region {
            let entity = create_entity_in_partition(
                runtime,
                &format!("{label}-region-{region}-node-{node}"),
                partition_id,
            );
            entities.push(entity);
        }

        let region_end = entities.len();
        let region_entities = &entities[region_start..region_end];
        for index in 0..(region_entities.len().saturating_sub(1)) {
            create_relation_in_partition(
                runtime,
                region_entities[index],
                region_entities[index + 1],
                &format!("{label}-region-{region}-chain-{index}"),
                PartitionId(760 + region as u32),
            );
            relation_count += 1;
        }

        for index in (0..region_entities.len().saturating_sub(3)).step_by(3) {
            create_relation_in_partition(
                runtime,
                region_entities[index],
                region_entities[index + 3],
                &format!("{label}-region-{region}-skip-{index}"),
                PartitionId(820 + region as u32),
            );
            relation_count += 1;
        }

        let actor = region_entities[region_entities.len() / 2];
        frame_targets.push(actor);
        explicit_targets.push(region_entities[1]);
        explicit_targets.push(region_entities[region_entities.len() - 2]);
        if region < 6 {
            propagation_seeds.push(actor);
        }
    }

    for region in 0..region_count {
        let current = frame_targets[region];
        let next = frame_targets[(region + 1) % region_count];
        create_relation_in_partition(
            runtime,
            current,
            next,
            &format!("{label}-region-downstream-{region}"),
            PartitionId(900 + region as u32),
        );
        relation_count += 1;
    }

    GameEngineFrameSeedOutcome {
        entities,
        frame_targets,
        explicit_targets,
        propagation_seeds,
        relation_count,
        region_count,
    }
}
