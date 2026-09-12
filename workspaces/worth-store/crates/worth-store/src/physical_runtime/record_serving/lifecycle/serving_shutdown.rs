use crate::physical_runtime::MediaShutdownOutcome;

use super::super::{RecordPublicationResidueObservation, RecordServingCounterSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordServingTerminalPosture {
    NoInspectionRequired,
    InspectionRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordServingOwnerDisposition {
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordServingTerminalObservation {
    posture: RecordServingTerminalPosture,
    owner: RecordServingOwnerDisposition,
    residue: RecordPublicationResidueObservation,
    counters: RecordServingCounterSnapshot,
}

impl RecordServingTerminalObservation {
    pub(in crate::physical_runtime) fn new(
        inspection_required: bool,
        residue: RecordPublicationResidueObservation,
        counters: RecordServingCounterSnapshot,
    ) -> Self {
        assert_eq!(
            counters.owner_live(),
            0,
            "record-owner release must precede terminal observation"
        );
        Self {
            posture: if inspection_required {
                RecordServingTerminalPosture::InspectionRequired
            } else {
                RecordServingTerminalPosture::NoInspectionRequired
            },
            owner: RecordServingOwnerDisposition::Released,
            residue,
            counters,
        }
    }

    pub const fn posture(self) -> RecordServingTerminalPosture {
        self.posture
    }

    pub const fn owner(self) -> RecordServingOwnerDisposition {
        self.owner
    }

    pub const fn residue(self) -> RecordPublicationResidueObservation {
        self.residue
    }

    pub const fn counters(self) -> RecordServingCounterSnapshot {
        self.counters
    }
}

pub struct ServingShutdownOutcome<Terminal> {
    pub(in crate::physical_runtime) read_protection:
        crate::physical_runtime::PhysicalReadProtectionShutdown,
    pub(in crate::physical_runtime) media: MediaShutdownOutcome<Terminal>,
    pub(in crate::physical_runtime) records: RecordServingTerminalObservation,
    pub(in crate::physical_runtime) mutation: crate::physical_runtime::PhysicalMutationShutdown,
    pub(in crate::physical_runtime) checkpoint: crate::physical_runtime::PhysicalCheckpointShutdown,
    pub(in crate::physical_runtime) residency: worth_store_buffer_pool::PhysicalResidencyShutdown,
    pub(in crate::physical_runtime) work: crate::physical_runtime::PhysicalWorkShutdownObservation,
    pub(in crate::physical_runtime) signal: crate::physical_runtime::PhysicalSignalShutdownOutcome,
    pub(in crate::physical_runtime) signal_summary:
        Option<worth_signal::facade::ResourceRuntimeSummary>,
    pub(in crate::physical_runtime) signal_cancellation_failures: u64,
    pub(in crate::physical_runtime) durability_closeout:
        crate::physical_runtime::PhysicalDurabilityCloseoutOutcome,
    pub(in crate::physical_runtime) performance:
        crate::physical_runtime::PhysicalDurabilityPerformanceSummary,
}

impl<Terminal> ServingShutdownOutcome<Terminal> {
    pub const fn read_protection(&self) -> crate::physical_runtime::PhysicalReadProtectionShutdown {
        self.read_protection
    }
    pub const fn terminal(&self) -> &Terminal {
        self.media.terminal()
    }

    pub const fn media(&self) -> &MediaShutdownOutcome<Terminal> {
        &self.media
    }

    pub const fn records(&self) -> RecordServingTerminalObservation {
        self.records
    }

    pub const fn mutations(&self) -> crate::physical_runtime::PhysicalMutationShutdown {
        self.mutation
    }

    pub const fn checkpoint(&self) -> crate::physical_runtime::PhysicalCheckpointShutdown {
        self.checkpoint
    }

    pub const fn residency(&self) -> worth_store_buffer_pool::PhysicalResidencyShutdown {
        self.residency
    }

    pub const fn work(&self) -> &crate::physical_runtime::PhysicalWorkShutdownObservation {
        &self.work
    }

    pub const fn signal(&self) -> crate::physical_runtime::PhysicalSignalShutdownOutcome {
        self.signal
    }

    pub const fn signal_summary(&self) -> Option<worth_signal::facade::ResourceRuntimeSummary> {
        self.signal_summary
    }

    pub const fn signal_cancellation_failures(&self) -> u64 {
        self.signal_cancellation_failures
    }

    pub const fn durability_closeout(
        &self,
    ) -> &crate::physical_runtime::PhysicalDurabilityCloseoutOutcome {
        &self.durability_closeout
    }

    pub(in crate::physical_runtime) fn into_durability_closeout(
        self,
    ) -> crate::physical_runtime::PhysicalDurabilityCloseoutOutcome {
        self.durability_closeout
    }

    pub const fn performance(
        &self,
    ) -> crate::physical_runtime::PhysicalDurabilityPerformanceSummary {
        self.performance
    }
}
