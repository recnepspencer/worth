#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPlanCostDenial {
    RedoTargets,
    RedoBytes,
    DistinctTargets,
    OperationBindings,
    ObservationBytes,
    StagingBytes,
    RecoveryMemoryBytes,
    DirtyFrames,
}
