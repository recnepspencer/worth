//! Conservative live-byte ledger for an addressed source inventory.

use worth_store_physical_format::PhysicalRecordFormatDeclaration;

use super::PageObservationFailure;

pub(in crate::orchestration::planning) struct ResidentAllowance {
    used: u64,
    maximum: u64,
    peak: u64,
    required: u64,
}

impl ResidentAllowance {
    pub(in crate::orchestration::planning) const fn new(maximum: u64) -> Self {
        Self {
            used: 0,
            maximum,
            peak: 0,
            required: 0,
        }
    }

    pub(in crate::orchestration::planning) const fn used(&self) -> u64 {
        self.used
    }

    pub(in crate::orchestration::planning) const fn remaining(&self) -> u64 {
        self.maximum.saturating_sub(self.used)
    }

    pub(in crate::orchestration::planning) const fn peak(&self) -> u64 {
        self.peak
    }

    pub(in crate::orchestration::planning) const fn exceeded_requirement(&self) -> Option<u64> {
        if self.required > self.maximum {
            Some(self.required)
        } else {
            None
        }
    }

    /// Admit a concurrently live window without retaining it after the caller
    /// drops its scratch. Every subsequent allocation still debits this ledger.
    pub(in crate::orchestration::planning) fn transient(
        &mut self,
        bytes: u64,
    ) -> Result<(), PageObservationFailure> {
        let Some(required) = self.used.checked_add(bytes) else {
            self.required = u64::MAX;
            return Err(PageObservationFailure::ManifestEntryLimit);
        };
        self.required = self.required.max(required);
        if required > self.maximum {
            return Err(PageObservationFailure::ManifestEntryLimit);
        }
        self.peak = self.peak.max(required);
        Ok(())
    }

    pub(in crate::orchestration::planning) fn bytes(
        &mut self,
        bytes: u64,
    ) -> Result<(), PageObservationFailure> {
        self.charge(bytes)
    }

    pub(in crate::orchestration::planning) fn trace_slots(
        &mut self,
        trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
        additional: usize,
    ) -> Result<(), PageObservationFailure> {
        // Old backing is already retained in used. A growth allocation can
        // coexist with it; preflight the complete new backing, not just delta.
        let allocation = trace
            .observation_reservation_bytes(additional)
            .ok_or(PageObservationFailure::ManifestEntryLimit)?;
        self.transient(allocation)?;
        let growth = trace
            .reserve_observations(additional)
            .ok_or(PageObservationFailure::ManifestEntryLimit)?;
        self.bytes(growth)
    }

    pub(in crate::orchestration::planning) fn release(
        &mut self,
        bytes: u64,
    ) -> Result<(), PageObservationFailure> {
        self.used = self
            .used
            .checked_sub(bytes)
            .ok_or(PageObservationFailure::ManifestEntryLimit)?;
        Ok(())
    }

    pub(in crate::orchestration::planning) fn block(
        &mut self,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), PageObservationFailure> {
        // The decoded block, topology clone, traversal queue, visited set,
        // and artifact set can all coexist until the complete root is read.
        self.charge(
            u64::from(format.page_size().bytes())
                .checked_mul(4)
                .and_then(|value| value.checked_add(512))
                .ok_or(PageObservationFailure::ManifestEntryLimit)?,
        )
    }

    pub(in crate::orchestration::planning) fn entries(
        &mut self,
        count: usize,
        entry_bytes: usize,
    ) -> Result<(), PageObservationFailure> {
        self.charge(
            u64::try_from(count)
                .ok()
                .and_then(|count| {
                    u64::try_from(entry_bytes)
                        .ok()
                        .and_then(|bytes| bytes.checked_mul(2))
                        .and_then(|bytes| bytes.checked_add(8 * 8))
                        .and_then(|bytes| count.checked_mul(bytes))
                })
                .ok_or(PageObservationFailure::ManifestEntryLimit)?,
        )
    }

    fn charge(&mut self, bytes: u64) -> Result<(), PageObservationFailure> {
        self.transient(bytes)?;
        self.used = self
            .used
            .checked_add(bytes)
            .ok_or(PageObservationFailure::ManifestEntryLimit)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_free_heavy_inventory_denies_before_second_allocation() {
        let mut resident = ResidentAllowance::new(16_000);
        resident.entries(100, 32).unwrap();
        assert_eq!(resident.used(), 12_800);
        assert!(matches!(
            resident.entries(100, 32),
            Err(PageObservationFailure::ManifestEntryLimit)
        ));
    }

    #[test]
    fn scratch_peak_preserves_retained_charge_and_denies_one_byte_over() {
        let mut resident = ResidentAllowance::new(1_000);
        resident.bytes(300).unwrap();
        resident.transient(700).unwrap();
        assert_eq!(resident.used(), 300);
        assert_eq!(resident.peak(), 1_000);
        assert!(resident.transient(701).is_err());
        assert_eq!(resident.used(), 300);
        assert_eq!(resident.exceeded_requirement(), Some(1_001));
        resident.release(300).unwrap();
        assert_eq!(resident.used(), 0);
        assert_eq!(resident.peak(), 1_000);
    }

    #[test]
    fn overflowing_scratch_charge_is_rejected_even_at_maximum_ceiling() {
        let mut resident = ResidentAllowance::new(u64::MAX);
        resident.bytes(1).unwrap();
        assert!(resident.transient(u64::MAX).is_err());
        assert_eq!(resident.used(), 1);
    }
}
