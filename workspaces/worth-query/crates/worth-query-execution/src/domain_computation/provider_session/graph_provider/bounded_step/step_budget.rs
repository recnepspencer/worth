use worth_query_installation::facade::WorthQueryInstalledBoundedStepContract;

use super::{WorthQueryGraphProviderStepDenial, WorthQueryGraphProviderStepDenialKind};

pub(super) struct WorthQueryGraphProviderStepBudget {
    max_work_units: u64,
    completed_work_units: u64,
    chunk_width_ceiling: u64,
    scratch_bytes_ceiling: u64,
    retained_bytes_ceiling: u64,
    peak_scratch_bytes: u64,
    retained_bytes: u64,
}

impl WorthQueryGraphProviderStepBudget {
    pub(super) fn new(
        contract: &WorthQueryInstalledBoundedStepContract,
        managed_retained_bytes: u64,
    ) -> Self {
        Self {
            max_work_units: contract.max_work_units_per_step(),
            completed_work_units: 0,
            chunk_width_ceiling: contract.chunk_width_ceiling(),
            scratch_bytes_ceiling: contract.scratch_bytes_ceiling(),
            retained_bytes_ceiling: contract.retained_bytes_ceiling(),
            peak_scratch_bytes: 0,
            retained_bytes: managed_retained_bytes,
        }
    }

    pub(super) const fn remaining_work_units(&self) -> u64 {
        self.max_work_units
            .saturating_sub(self.completed_work_units)
    }

    pub(super) fn admit_work_unit(&self) -> Result<(), WorthQueryGraphProviderStepDenial> {
        if self.completed_work_units >= self.max_work_units {
            return Err(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::WorkBudgetExceeded,
                "provider step work exceeds the installed work budget",
            ));
        }
        Ok(())
    }

    pub(super) fn complete_work_unit(&mut self) {
        debug_assert!(self.completed_work_units < self.max_work_units);
        self.completed_work_units += 1;
    }

    pub(super) fn validate_chunk_width(
        &self,
        width: usize,
    ) -> Result<(), WorthQueryGraphProviderStepDenial> {
        if u64::try_from(width).unwrap_or(u64::MAX) > self.chunk_width_ceiling {
            return Err(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::ChunkWidthExceeded,
                "provider projection chunk exceeds the installed chunk width",
            ));
        }
        Ok(())
    }

    pub(super) fn validate_scratch(
        &self,
        scratch_bytes: u64,
    ) -> Result<(), WorthQueryGraphProviderStepDenial> {
        if scratch_bytes > self.scratch_bytes_ceiling {
            return Err(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::ScratchBudgetExceeded,
                "provider step scratch exceeds the installed scratch budget",
            ));
        }
        Ok(())
    }

    pub(super) fn admit_scratch(
        &mut self,
        scratch_bytes: u64,
    ) -> Result<(), WorthQueryGraphProviderStepDenial> {
        self.validate_scratch(scratch_bytes)?;
        self.peak_scratch_bytes = self.peak_scratch_bytes.max(scratch_bytes);
        Ok(())
    }

    pub(super) fn admit_retained_component(
        &mut self,
        retained_bytes: u64,
    ) -> Result<(), WorthQueryGraphProviderStepDenial> {
        let total = self
            .retained_bytes
            .checked_add(retained_bytes)
            .ok_or_else(|| {
                WorthQueryGraphProviderStepDenial::new(
                    WorthQueryGraphProviderStepDenialKind::RetainedBudgetExceeded,
                    "provider step retained memory exceeds the installed retained budget",
                )
            })?;
        if total > self.retained_bytes_ceiling {
            return Err(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::RetainedBudgetExceeded,
                "provider step retained memory exceeds the installed retained budget",
            ));
        }
        self.retained_bytes = total;
        Ok(())
    }

    pub(super) const fn completed_work_units(&self) -> u64 {
        self.completed_work_units
    }

    pub(super) const fn peak_scratch_bytes(&self) -> u64 {
        self.peak_scratch_bytes
    }
}
