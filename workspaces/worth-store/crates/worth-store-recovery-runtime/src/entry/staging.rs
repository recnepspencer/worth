use worth_store::physical_runtime::{
    CompletedPhysicalRecoveryStagingCommand, PhysicalRecoveryStagingCommandDenial,
    PhysicalRecoveryStagingCommandIndeterminate, PhysicalRecoveryStagingCommandStage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalRecoveryStagingCounters {
    pub planned_scheduler_commands: u64,
    pub commands_submitted: u64,
    pub commands_settled: u64,
    pub scheduler_settlements: u64,
    pub artifacts_created: u64,
    pub artifacts_converged: u64,
    pub artifacts_completed_from_prefix: u64,
    pub artifacts_synchronized: u64,
    pub bytes_written: u64,
    pub bytes_verified: u64,
    pub performed_effects: u64,
    pub live_commands_after_close: u64,
    pub live_scheduler_reservations_after_close: u64,
    pub pending_signal_reconciliations_after_close: u64,
    pub signal_reconciliation_overflow_after_close: u64,
    pub live_media_handles_after_close: u64,
}

pub struct PhysicalRecoveryStagingSettlementLedger {
    entries: Box<[PhysicalRecoveryStagingSettlement]>,
}

pub enum PhysicalRecoveryStagingSettlement {
    Completed(CompletedPhysicalRecoveryStagingCommand),
    DeniedBeforeEffect(PhysicalRecoveryStagingCommandDenial),
    Indeterminate(PhysicalRecoveryStagingCommandIndeterminate),
}

impl std::fmt::Debug for PhysicalRecoveryStagingSettlementLedger {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PhysicalRecoveryStagingSettlementLedger")
            .field("entries", &self.entries.len())
            .field("completed", &self.completed())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryStagingDenial {
    InvalidPlan,
    CommandFailed {
        ordinal: u64,
        stage: PhysicalRecoveryStagingCommandStage,
    },
    Indeterminate {
        ordinal: u64,
        stage: PhysicalRecoveryStagingCommandStage,
    },
    CancelledAfterClosedStaging,
    CancelledAfterPartialStaging {
        settled_commands: u64,
    },
    QuiescenceMismatch,
}

impl PhysicalRecoveryStagingSettlementLedger {
    /// Retained settlement backing and Store-owned nested effect storage.
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let entries = u64::try_from(std::mem::size_of_val(&*self.entries)).ok()?;
        self.entries.iter().try_fold(entries, |total, entry| {
            let nested = match entry {
                PhysicalRecoveryStagingSettlement::Completed(value) => value.owned_heap_bytes()?,
                PhysicalRecoveryStagingSettlement::DeniedBeforeEffect(value) => {
                    value.owned_heap_bytes()?
                }
                PhysicalRecoveryStagingSettlement::Indeterminate(value) => {
                    value.owned_heap_bytes()?
                }
            };
            total.checked_add(nested)
        })
    }

    pub(crate) fn new(entries: Vec<PhysicalRecoveryStagingSettlement>) -> Self {
        Self {
            entries: entries.into_boxed_slice(),
        }
    }
    pub fn entries(&self) -> &[PhysicalRecoveryStagingSettlement] {
        &self.entries
    }
    pub fn completed(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| matches!(entry, PhysicalRecoveryStagingSettlement::Completed(_)))
            .count()
    }
}
