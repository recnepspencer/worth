mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::telemetry::{
    CheckpointTelemetry, EvaluationTelemetry, ExecutionTelemetry, HostComputedTelemetry,
    InvalidationTelemetry, PlannerTelemetry, RuntimeTelemetry, StorageTelemetry, TemporalTelemetry,
    TransactionTelemetry,
};

/// Read-only summary of graph-local runtime telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GraphMetrics {
    pub evaluation: EvaluationTelemetry,
    pub invalidation: InvalidationTelemetry,
    pub planner: PlannerTelemetry,
    pub execution: ExecutionTelemetry,
    pub storage: StorageTelemetry,
    pub temporal: TemporalTelemetry,
    pub host_computed: HostComputedTelemetry,
    pub partition_interner_size: usize,
}

impl GraphMetrics {
    pub fn from_runtime_telemetry(
        telemetry: &RuntimeTelemetry,
        partition_interner_size: usize,
    ) -> Self {
        Self {
            evaluation: telemetry.evaluation,
            invalidation: telemetry.invalidation,
            planner: telemetry.planner,
            execution: telemetry.execution,
            storage: telemetry.storage,
            temporal: telemetry.temporal,
            host_computed: telemetry.host_computed,
            partition_interner_size,
        }
    }
}

/// Read-only summary of runtime-level orchestration telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RuntimeMetrics {
    pub evaluation: EvaluationTelemetry,
    pub invalidation: InvalidationTelemetry,
    pub transaction: TransactionTelemetry,
    pub planner: PlannerTelemetry,
    pub execution: ExecutionTelemetry,
    pub storage: StorageTelemetry,
    pub checkpoint: CheckpointTelemetry,
    pub temporal: TemporalTelemetry,
    pub host_computed: HostComputedTelemetry,
}
