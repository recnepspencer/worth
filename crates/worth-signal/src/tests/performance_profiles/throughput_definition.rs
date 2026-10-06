use std::time::{Duration, Instant};

use crate::facade::SignalRuntimePolicy;
use crate::tests::domains::fintech::{
    compile_financial_locality_world_with_policy, FinancialWorldDefinition,
};
use worth_foundational::{ExecutionObjectiveProfile, ObservationActivationProfile};

pub(crate) const PERFORMANCE_SEED: u64 = 41;
pub(super) const ORDINARY_OUTPUT_FLOOR: u32 = 1_024;
/// Largest scheduled output floor this environment can settle inside 10 minutes.
pub(super) const RECORDED_SCHEDULED_OUTPUT_FLOOR: u32 = 4_096;
pub(super) const PERFORMANCE_BATCHES: usize = 120;
pub(super) const THROUGHPUT_TEST_BUDGET: Duration = Duration::from_secs(600);

#[derive(Debug)]
pub(super) struct PerformancePacketContext {
    pub(super) seed: u64,
    pub(super) principal_nodes: usize,
    pub(super) scheduled_node_bound: usize,
    pub(super) batches: usize,
    pub(super) batch_width: &'static str,
    pub(super) trial_count: usize,
    pub(super) hardware_os: &'static str,
    pub(super) hardware_arch: &'static str,
    pub(super) build_profile: &'static str,
    pub(super) thread_count: usize,
    pub(super) thread_posture: &'static str,
    pub(super) feature_posture: &'static str,
}

impl PerformancePacketContext {
    pub(super) fn recorded() -> Self {
        Self {
            seed: PERFORMANCE_SEED,
            principal_nodes: ORDINARY_OUTPUT_FLOOR as usize,
            scheduled_node_bound: RECORDED_SCHEDULED_OUTPUT_FLOOR as usize,
            batches: PERFORMANCE_BATCHES,
            batch_width: "approximately one percent; broad=0.5%-5%, narrow<=5%",
            trial_count: 1,
            hardware_os: std::env::consts::OS,
            hardware_arch: std::env::consts::ARCH,
            build_profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            thread_count: std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(1),
            thread_posture: "host-lease-automatic-four-workers",
            feature_posture: "default-shared-execution",
        }
    }
}

impl std::fmt::Display for PerformancePacketContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "seed={} principal_nodes={} scheduled_node_bound={} batches={} batch_width={} trial_count={} os={} arch={} profile={} threads={} thread_posture={} features={}",
            self.seed,
            self.principal_nodes,
            self.scheduled_node_bound,
            self.batches,
            self.batch_width,
            self.trial_count,
            self.hardware_os,
            self.hardware_arch,
            self.build_profile,
            self.thread_count,
            self.thread_posture,
            self.feature_posture,
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PerformanceProfile {
    pub(crate) name: &'static str,
    pub(crate) policy: SignalRuntimePolicy,
    pub(crate) explicit_observation: bool,
}

impl PerformanceProfile {
    pub(crate) fn expects_optional_observation(self) -> bool {
        self.explicit_observation
            || self.policy.observation_activation == ObservationActivationProfile::Continuous
    }
}

pub(crate) fn profiles() -> [PerformanceProfile; 6] {
    [
        PerformanceProfile {
            name: "balanced_continuous",
            policy: SignalRuntimePolicy::operational()
                .with_execution_objective(ExecutionObjectiveProfile::Balanced)
                .with_observation_activation(ObservationActivationProfile::Continuous),
            explicit_observation: false,
        },
        PerformanceProfile {
            name: "throughput_idle",
            policy: SignalRuntimePolicy::operational(),
            explicit_observation: false,
        },
        PerformanceProfile {
            name: "throughput_observed",
            policy: SignalRuntimePolicy::operational()
                .with_observation_activation(ObservationActivationProfile::OnDemand),
            explicit_observation: true,
        },
        PerformanceProfile {
            name: "throughput_rich_session",
            policy: SignalRuntimePolicy::forensic()
                .with_execution_objective(ExecutionObjectiveProfile::Throughput)
                .with_observation_activation(ObservationActivationProfile::OnDemand),
            explicit_observation: true,
        },
        PerformanceProfile {
            name: "introspective",
            policy: SignalRuntimePolicy::forensic()
                .with_execution_objective(ExecutionObjectiveProfile::Balanced)
                .with_observation_activation(ObservationActivationProfile::Continuous),
            explicit_observation: false,
        },
        PerformanceProfile {
            name: "latency_bounded",
            policy: SignalRuntimePolicy::operational()
                .with_execution_objective(ExecutionObjectiveProfile::LatencyBounded)
                .with_observation_activation(ObservationActivationProfile::OnDemand),
            explicit_observation: false,
        },
    ]
}

pub(super) fn ordinary_definition() -> FinancialWorldDefinition {
    partitioned_world_for_output_floor(PERFORMANCE_SEED, ORDINARY_OUTPUT_FLOOR)
}

pub(super) fn partitioned_world_for_output_floor(
    seed: u64,
    min_outputs: u32,
) -> FinancialWorldDefinition {
    // PartitionedCurveUniverse emits 7 + 2*(regions-1) outputs when memberships=1
    // and instruments=1. Size regions to the claimed floor instead of a larger
    // generator whose node count only happens to exceed the floor.
    let regions = u16::try_from(min_outputs.saturating_sub(5).div_ceil(2).max(1))
        .expect("throughput output floor must fit the partitioned region axis");
    FinancialWorldDefinition::partitioned_curve_universe_performance(seed, regions, 1, 1)
}

pub(super) fn operational_digest_for(
    profile: PerformanceProfile,
    definition: FinancialWorldDefinition,
    batches: usize,
) -> (
    crate::tests::domains::fintech::FinancialPerformanceBatchReport,
    worth_foundational::facade::CanonicalDigestId,
    crate::tests::domains::fintech::LocalityOptionalObservationInventory,
) {
    let mut world = compile_financial_locality_world_with_policy(definition, profile.policy)
        .expect("throughput digest world compiles");
    let report = world
        .run_locality_performance_sequence(
            batches,
            performance_workers(),
            profile.explicit_observation,
        )
        .expect("throughput digest sequence settles");
    let digest = world
        .locality_operational_digest_without_observation_work()
        .expect("semantic operational digest derives");
    let inventory = world.locality_optional_observation_inventory();
    (report, digest, inventory)
}

pub(super) fn assert_profile_report(
    report: &crate::tests::domains::fintech::FinancialPerformanceBatchReport,
    min_nodes: usize,
    batches: usize,
) {
    assert!(report.node_count >= min_nodes);
    assert_eq!(report.batch_count, batches);
    assert!(report.warm_median_micros > 0);
    assert!(report.warm_p95_micros >= report.warm_median_micros);
    assert!(report.completed_batches_per_second > 0);
    assert!(report.semantic_work_rows.iter().all(|row| !row.is_empty()));
    assert_eq!(report.semantic_work_rows.len(), batches);
    assert_eq!(report.mutation_widths.len(), batches);
    for (batch, (row, mutation_width)) in report
        .semantic_work_rows
        .iter()
        .zip(&report.mutation_widths)
        .enumerate()
    {
        assert!(
            *mutation_width >= report.node_count / 200
                && *mutation_width <= report.node_count / 20,
            "batch {batch} declared edit width escaped the one-percent band: {mutation_width} of {}",
            report.node_count
        );
        let broad_scope_edit = batch % 3 != 2;
        let in_band = if broad_scope_edit {
            row.len() >= report.node_count / 200 && row.len() <= report.node_count / 20
        } else {
            row.len() <= report.node_count / 20
        };
        assert!(
            in_band,
            "batch {batch} escaped its declared locality band: {} of {}",
            row.len(),
            report.node_count
        );
    }
}

pub(crate) fn assert_within_throughput_budget(started: Instant, label: &str) {
    let elapsed = started.elapsed();
    assert!(
        elapsed <= THROUGHPUT_TEST_BUDGET,
        "{label} exceeded the 10-minute throughput test budget: {elapsed:?}"
    );
}

/// Requested placement; physical dispatch is read from the actual authority report.
pub(super) fn performance_workers() -> usize {
    4
}
