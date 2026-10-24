mod operation_application;
mod operation_selection;
mod property_runtime;
mod scenario_execution;

pub(crate) use property_runtime::build_property_runtime;
pub(crate) use scenario_execution::{run_property_scenario, run_seeded_scenario};

use crate::facade::config::RelationalRuntimeProfile;
use crate::facade::history::BranchId;
use crate::facade::identity::{EntityId, PartitionId, RelationId};
use crate::facade::publication::SubscriberCheckpoint;
use crate::facade::runtime::RelationalRuntime;

use super::operation::ScenarioOperation;
use super::profiles::CertificationPressureProfile;
use crate::tests::harness::fixtures::runtime::RuntimeHarnessMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ActiveRelation {
    pub(super) relation_id: RelationId,
    pub(super) source: EntityId,
    pub(super) target: EntityId,
    pub(super) partition: PartitionId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SeededScenarioConfig {
    pub(crate) seed: u64,
    pub(crate) steps: usize,
    pub(crate) checkpoint_stride: usize,
    pub(crate) runtime_mode: RuntimeHarnessMode,
    pub(crate) relation_pressure: bool,
    pub(crate) durable_checkpoint_every: Option<usize>,
    pub(crate) retention_pass_every: Option<usize>,
    pub(crate) branch_pressure: bool,
    pub(crate) replacement_pressure: bool,
}

impl SeededScenarioConfig {
    pub(crate) fn geometry_kernel(seed: u64, profile: CertificationPressureProfile) -> Self {
        Self {
            seed,
            steps: profile.steps(),
            checkpoint_stride: 16,
            runtime_mode: RuntimeHarnessMode::InMemory(RelationalRuntimeProfile::GeometryKernel),
            relation_pressure: true,
            durable_checkpoint_every: None,
            retention_pass_every: None,
            branch_pressure: false,
            replacement_pressure: false,
        }
    }

    pub(crate) fn persisted_geometry(seed: u64, profile: CertificationPressureProfile) -> Self {
        Self {
            seed,
            steps: profile.steps(),
            checkpoint_stride: 16,
            runtime_mode: RuntimeHarnessMode::Persisted,
            relation_pressure: true,
            durable_checkpoint_every: Some(32),
            retention_pass_every: Some(8),
            branch_pressure: false,
            replacement_pressure: true,
        }
    }

    pub(crate) fn hostile_geometry(seed: u64, profile: CertificationPressureProfile) -> Self {
        Self {
            seed,
            steps: profile.steps(),
            checkpoint_stride: 8,
            runtime_mode: RuntimeHarnessMode::InMemory(RelationalRuntimeProfile::GeometryKernel),
            relation_pressure: true,
            durable_checkpoint_every: None,
            retention_pass_every: Some(4),
            branch_pressure: true,
            replacement_pressure: true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ScenarioTrace {
    pub(crate) seed: u64,
    pub(crate) operations: Vec<ScenarioOperation>,
}

#[derive(Debug)]
pub(crate) struct SeededScenarioWorld {
    pub(crate) runtime: RelationalRuntime,
    pub(crate) baseline_checkpoint: SubscriberCheckpoint,
    pub(crate) checkpoints: Vec<SubscriberCheckpoint>,
    pub(crate) trace: ScenarioTrace,
}

pub(super) fn scenario_branch_main() -> BranchId {
    BranchId("main".to_string())
}
