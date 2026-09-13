use std::sync::atomic::{AtomicU64, Ordering};

use super::{PhysicalWorkOperationFamily, PhysicalWorkPressureClass, PhysicalWorkTerminalStage};

pub(in crate::physical_runtime::work) struct PhysicalWorkAccounting {
    declared: AtomicU64,
    terminal_by_family_and_pressure: [[AtomicU64; 8]; 9],
}

impl PhysicalWorkAccounting {
    pub(in crate::physical_runtime::work) const fn new() -> Self {
        Self {
            declared: AtomicU64::new(0),
            terminal_by_family_and_pressure: [const { [const { AtomicU64::new(0) }; 8] }; 9],
        }
    }

    pub(in crate::physical_runtime::work) fn record_declared(&self) {
        let _ = self
            .declared
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            });
    }

    pub(in crate::physical_runtime::work) fn declared(&self) -> u64 {
        self.declared.load(Ordering::Acquire)
    }

    pub(in crate::physical_runtime::work) fn record_terminal(
        &self,
        family: PhysicalWorkOperationFamily,
        pressure: PhysicalWorkPressureClass,
    ) {
        let _ = self.terminal_by_family_and_pressure[family_index(family)]
            [pressure_index(pressure)]
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        });
    }

    pub(in crate::physical_runtime::work) fn terminal_by_family_and_pressure(
        &self,
    ) -> [[u64; 8]; 9] {
        std::array::from_fn(|family| {
            std::array::from_fn(|pressure| {
                self.terminal_by_family_and_pressure[family][pressure].load(Ordering::Acquire)
            })
        })
    }

    pub(in crate::physical_runtime::work) fn terminal(&self) -> u64 {
        self.terminal_by_family_and_pressure()
            .iter()
            .flatten()
            .copied()
            .sum()
    }
}

pub(in crate::physical_runtime::work) const fn family_index(
    family: PhysicalWorkOperationFamily,
) -> usize {
    match family {
        PhysicalWorkOperationFamily::ArtifactMetadataRead => 0,
        PhysicalWorkOperationFamily::ArtifactRangeRead => 1,
        PhysicalWorkOperationFamily::ArtifactRangeWrite => 2,
        PhysicalWorkOperationFamily::ArtifactPublication => 3,
        PhysicalWorkOperationFamily::CheckpointCapture => 4,
        PhysicalWorkOperationFamily::WalAppend => 5,
        PhysicalWorkOperationFamily::DurabilityBarrier => 6,
        PhysicalWorkOperationFamily::WalReclamation => 7,
        PhysicalWorkOperationFamily::RootPublication => 8,
    }
}

pub(in crate::physical_runtime::work) const fn terminal_stage_index(
    stage: PhysicalWorkTerminalStage,
) -> usize {
    match stage {
        PhysicalWorkTerminalStage::Declared => 0,
        PhysicalWorkTerminalStage::Blocked => 1,
        PhysicalWorkTerminalStage::Ready => 2,
        PhysicalWorkTerminalStage::Queued => 3,
        PhysicalWorkTerminalStage::Dispatched => 4,
        PhysicalWorkTerminalStage::Settling => 5,
    }
}

pub(in crate::physical_runtime::work) const fn pressure_index(
    pressure: PhysicalWorkPressureClass,
) -> usize {
    match pressure {
        PhysicalWorkPressureClass::Unscheduled => 0,
        PhysicalWorkPressureClass::ForegroundPointRead => 1,
        PhysicalWorkPressureClass::ForegroundRangeRead => 2,
        PhysicalWorkPressureClass::ForegroundInteractiveRead => 3,
        PhysicalWorkPressureClass::ForegroundInternalRead => 4,
        PhysicalWorkPressureClass::ForegroundMutation => 5,
        PhysicalWorkPressureClass::BackgroundCheckpoint => 6,
        PhysicalWorkPressureClass::BackgroundScrub => 7,
    }
}
