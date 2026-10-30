#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentWriterDurabilityProfileSelection {
    CheckpointWritebackV1,
    CleanupRotationV1,
}

impl ParentWriterDurabilityProfileSelection {
    pub(crate) const fn cli_name(self) -> &'static str {
        match self {
            Self::CheckpointWritebackV1 => "c8-phase8-checkpoint-writeback-v1",
            Self::CleanupRotationV1 => "c8-phase8-cleanup-rotation-v1",
        }
    }
}
