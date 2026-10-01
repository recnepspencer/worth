use std::num::NonZeroU64;

use crate::physical_runtime::{
    layout::{PhysicalLayoutMaintenanceFailure, PhysicalLayoutPagePort},
    ServingPhysicalRuntime,
};

use super::super::MAXIMUM_RETIREMENT_RECORDS;

/// The protected reader routes every old and newly appended node that can
/// participate in this retirement. Its count bounds each closure and their
/// unique union, independently of the lifetime number of retired nodes.
#[derive(Clone, Copy)]
pub(super) struct RetirementRecordBudget {
    maximum: usize,
}

impl RetirementRecordBudget {
    pub(super) fn from_port(port: &PhysicalLayoutPagePort<'_>) -> Self {
        let selected = usize::try_from(port.reader().selected_record_count()).unwrap_or(usize::MAX);
        Self {
            maximum: selected.min(MAXIMUM_RETIREMENT_RECORDS),
        }
    }

    pub(super) fn charge(
        self,
        runtime: &ServingPhysicalRuntime,
    ) -> Result<NonZeroU64, PhysicalLayoutMaintenanceFailure> {
        // BTreeSet nodes, a concurrent Vec/Arc copy, and decoded path nodes
        // may coexist. Preserve the established per-record and page geometry.
        (self.maximum as u64)
            .checked_mul(160)
            .and_then(|bytes| {
                u64::from(runtime.maximum_inline_record_bytes())
                    .checked_mul(8)
                    .and_then(|headroom| bytes.checked_add(headroom))
            })
            .and_then(NonZeroU64::new)
            .ok_or(PhysicalLayoutMaintenanceFailure::RetirementLimit)
    }

    pub(super) fn before_unique_insert(
        self,
        count: usize,
        already_present: bool,
    ) -> Result<(), PhysicalLayoutMaintenanceFailure> {
        if !already_present && count >= self.maximum {
            Err(PhysicalLayoutMaintenanceFailure::RetirementLimit)
        } else {
            Ok(())
        }
    }

    pub(super) fn before_copy(self, count: usize) -> Result<(), PhysicalLayoutMaintenanceFailure> {
        if count > self.maximum {
            Err(PhysicalLayoutMaintenanceFailure::RetirementLimit)
        } else {
            Ok(())
        }
    }

    #[cfg(test)]
    pub(super) const fn for_test(maximum: usize) -> Self {
        Self { maximum }
    }
}
