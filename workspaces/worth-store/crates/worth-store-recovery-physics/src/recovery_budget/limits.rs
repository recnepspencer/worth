#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryPlanLimits {
    redo_targets: u64,
    redo_bytes: u64,
    distinct_targets: u64,
    operation_bindings: u64,
    observation_bytes: u64,
    staging_bytes: u64,
    recovery_memory_bytes: u64,
    dirty_frames: u64,
}

impl RecoveryPlanLimits {
    pub const fn new(
        redo_targets: u64,
        redo_bytes: u64,
        distinct_targets: u64,
        operation_bindings: u64,
        observation_bytes: u64,
        staging_bytes: u64,
        recovery_memory_bytes: u64,
        dirty_frames: u64,
    ) -> Option<Self> {
        if redo_targets == 0
            || redo_bytes == 0
            || distinct_targets == 0
            || operation_bindings == 0
            || observation_bytes == 0
            || staging_bytes == 0
            || recovery_memory_bytes == 0
            || dirty_frames == 0
        {
            return None;
        }
        Some(Self {
            redo_targets,
            redo_bytes,
            distinct_targets,
            operation_bindings,
            observation_bytes,
            staging_bytes,
            recovery_memory_bytes,
            dirty_frames,
        })
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
    pub const fn observation_bytes(self) -> u64 {
        self.observation_bytes
    }
    pub const fn staging_bytes(self) -> u64 {
        self.staging_bytes
    }
    pub const fn recovery_memory_bytes(self) -> u64 {
        self.recovery_memory_bytes
    }
    pub const fn dirty_frames(self) -> u64 {
        self.dirty_frames
    }
}
