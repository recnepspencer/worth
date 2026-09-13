#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecoveryReportCounters {
    recovery_effects: u64,
    cleanup_performed: u64,
    cleanup_deferred: u64,
    peak_recovery_bytes: u64,
}

impl RecoveryReportCounters {
    pub(super) const fn new(
        recovery_effects: u64,
        cleanup_performed: u64,
        cleanup_deferred: u64,
        peak_recovery_bytes: u64,
    ) -> Self {
        Self {
            recovery_effects,
            cleanup_performed,
            cleanup_deferred,
            peak_recovery_bytes,
        }
    }

    pub const fn recovery_effects(self) -> u64 {
        self.recovery_effects
    }
    pub const fn cleanup_performed(self) -> u64 {
        self.cleanup_performed
    }
    pub const fn cleanup_deferred(self) -> u64 {
        self.cleanup_deferred
    }
    pub const fn peak_recovery_bytes(self) -> u64 {
        self.peak_recovery_bytes
    }
}
