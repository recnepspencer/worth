use super::{RecoveryPlanCostDenial, RecoveryPlanLimits};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryPlanCost {
    redo_targets: u64,
    redo_bytes: u64,
    distinct_targets: u64,
    operation_bindings: u64,
    observation_reads: u64,
    observation_bytes: u64,
    staging_bytes: u64,
    peak_recovery_bytes: u64,
    dirty_frames: u64,
}

pub const fn admit_recovery_plan_cost(
    limits: RecoveryPlanLimits,
    cost: RecoveryPlanCost,
) -> Result<RecoveryPlanCost, RecoveryPlanCostDenial> {
    if cost.redo_targets > limits.redo_targets() {
        return Err(RecoveryPlanCostDenial::RedoTargets);
    }
    if cost.redo_bytes > limits.redo_bytes() {
        return Err(RecoveryPlanCostDenial::RedoBytes);
    }
    if cost.distinct_targets > limits.distinct_targets() {
        return Err(RecoveryPlanCostDenial::DistinctTargets);
    }
    if cost.operation_bindings > limits.operation_bindings() {
        return Err(RecoveryPlanCostDenial::OperationBindings);
    }
    if cost.observation_bytes > limits.observation_bytes() {
        return Err(RecoveryPlanCostDenial::ObservationBytes);
    }
    if cost.staging_bytes > limits.staging_bytes() {
        return Err(RecoveryPlanCostDenial::StagingBytes);
    }
    if cost.peak_recovery_bytes > limits.recovery_memory_bytes() {
        return Err(RecoveryPlanCostDenial::RecoveryMemoryBytes);
    }
    if cost.dirty_frames > limits.dirty_frames() {
        return Err(RecoveryPlanCostDenial::DirtyFrames);
    }
    Ok(cost)
}

impl RecoveryPlanCost {
    pub const fn new(
        redo_targets: u64,
        redo_bytes: u64,
        distinct_targets: u64,
        operation_bindings: u64,
        observation_reads: u64,
        observation_bytes: u64,
        staging_bytes: u64,
        peak_recovery_bytes: u64,
        dirty_frames: u64,
    ) -> Self {
        Self {
            redo_targets,
            redo_bytes,
            distinct_targets,
            operation_bindings,
            observation_reads,
            observation_bytes,
            staging_bytes,
            peak_recovery_bytes,
            dirty_frames,
        }
    }
    pub const fn redo_targets(self) -> u64 {
        self.redo_targets
    }
    pub const fn redo_bytes(self) -> u64 {
        self.redo_bytes
    }
    pub const fn distinct_targets(self) -> u64 {
        self.distinct_targets
    }
    pub const fn operation_bindings(self) -> u64 {
        self.operation_bindings
    }
    pub const fn observation_reads(self) -> u64 {
        self.observation_reads
    }
    pub const fn observation_bytes(self) -> u64 {
        self.observation_bytes
    }
    pub const fn staging_bytes(self) -> u64 {
        self.staging_bytes
    }
    pub const fn peak_recovery_bytes(self) -> u64 {
        self.peak_recovery_bytes
    }
    pub const fn dirty_frames(self) -> u64 {
        self.dirty_frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_limit_is_admitted_and_each_one_over_limit_is_rejected() {
        let limits = RecoveryPlanLimits::new(2, 3, 2, 4, 5, 6, 7, 2).unwrap();
        let exact = RecoveryPlanCost::new(2, 3, 2, 4, 2, 5, 6, 7, 2);
        assert_eq!(admit_recovery_plan_cost(limits, exact), Ok(exact));

        let attacks = [
            (
                RecoveryPlanCost::new(3, 3, 2, 4, 2, 5, 6, 7, 2),
                RecoveryPlanCostDenial::RedoTargets,
            ),
            (
                RecoveryPlanCost::new(2, 4, 2, 4, 2, 5, 6, 7, 2),
                RecoveryPlanCostDenial::RedoBytes,
            ),
            (
                RecoveryPlanCost::new(2, 3, 3, 4, 2, 5, 6, 7, 2),
                RecoveryPlanCostDenial::DistinctTargets,
            ),
            (
                RecoveryPlanCost::new(2, 3, 2, 5, 2, 5, 6, 7, 2),
                RecoveryPlanCostDenial::OperationBindings,
            ),
            (
                RecoveryPlanCost::new(2, 3, 2, 4, 2, 6, 6, 7, 2),
                RecoveryPlanCostDenial::ObservationBytes,
            ),
            (
                RecoveryPlanCost::new(2, 3, 2, 4, 2, 5, 7, 7, 2),
                RecoveryPlanCostDenial::StagingBytes,
            ),
            (
                RecoveryPlanCost::new(2, 3, 2, 4, 2, 5, 6, 7, 3),
                RecoveryPlanCostDenial::DirtyFrames,
            ),
            (
                RecoveryPlanCost::new(2, 3, 2, 4, 2, 5, 6, 8, 2),
                RecoveryPlanCostDenial::RecoveryMemoryBytes,
            ),
        ];
        for (cost, expected) in attacks {
            assert_eq!(admit_recovery_plan_cost(limits, cost), Err(expected));
        }
    }
}
