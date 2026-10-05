//! Conservative live-byte ledger for an addressed source inventory.

use worth_store_physical_format::PhysicalRecordFormatDeclaration;

/// The allowance cannot hold what was asked of it. The ledger keeps what it
/// would have needed, so its owner reports its own limit with that value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::orchestration::planning) struct ResidentExhausted;

/// More bytes were handed back than the ledger holds. That is the caller's
/// accounting contradicting itself, never a limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::orchestration::planning) struct NotHeld;

#[derive(Debug)]
pub(in crate::orchestration::planning) enum ResidentTraceDenial {
    ResidentBoundExceeded,
    Allocation {
        requested: u64,
        cause: std::collections::TryReserveError,
    },
}

impl From<ResidentExhausted> for ResidentTraceDenial {
    fn from(_: ResidentExhausted) -> Self {
        Self::ResidentBoundExceeded
    }
}

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
    ) -> Result<(), ResidentExhausted> {
        self.window(bytes).map(|_| ())
    }

    pub(in crate::orchestration::planning) fn bytes(
        &mut self,
        bytes: u64,
    ) -> Result<(), ResidentExhausted> {
        self.used = self.window(bytes)?;
        Ok(())
    }

    /// What the ledger would hold with `bytes` more, if it admits that.
    fn window(&mut self, bytes: u64) -> Result<u64, ResidentExhausted> {
        let Some(required) = self.used.checked_add(bytes) else {
            return Err(self.unrepresentable());
        };
        self.required = self.required.max(required);
        if required > self.maximum {
            return Err(ResidentExhausted);
        }
        self.peak = self.peak.max(required);
        Ok(required)
    }

    /// A size no count of bytes can say is more than any allowance admits.
    fn unrepresentable(&mut self) -> ResidentExhausted {
        self.required = u64::MAX;
        ResidentExhausted
    }

    pub(in crate::orchestration::planning) fn trace_slots(
        &mut self,
        trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
        additional: usize,
    ) -> Result<(), ResidentTraceDenial> {
        // Old backing is already retained in used. A growth allocation can
        // coexist with it; preflight the complete new backing, not just delta.
        let (Some(allocation), Some(capacity), Some(old)) = (
            trace.observation_reservation_bytes(additional),
            trace.next_observation_capacity(additional),
            trace.owned_heap_bytes(),
        ) else {
            return Err(self.unrepresentable().into());
        };
        self.transient(allocation)?;
        trace
            .try_reserve_observation_capacity(capacity)
            .map_err(|cause| ResidentTraceDenial::Allocation {
                requested: allocation,
                cause,
            })?;
        // A reservation never shrinks the backing it grew.
        let actual = trace.owned_heap_bytes().unwrap_or(u64::MAX);
        Ok(self.bytes(actual.saturating_sub(old))?)
    }

    pub(in crate::orchestration::planning) fn release(
        &mut self,
        bytes: u64,
    ) -> Result<(), NotHeld> {
        self.used = self.used.checked_sub(bytes).ok_or(NotHeld)?;
        Ok(())
    }

    pub(in crate::orchestration::planning) fn block(
        &mut self,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), ResidentExhausted> {
        // The decoded block, topology clone, traversal queue, visited set,
        // and artifact set can all coexist until the complete root is read.
        self.bytes(u64::from(format.page_size().bytes()) * 4 + 512)
    }

    pub(in crate::orchestration::planning) fn entries(
        &mut self,
        count: usize,
        entry_bytes: usize,
    ) -> Result<(), ResidentExhausted> {
        let bytes = (entry_bytes as u64)
            .checked_mul(2)
            .and_then(|each| each.checked_add(8 * 8))
            .and_then(|each| each.checked_mul(count as u64));
        match bytes {
            Some(bytes) => self.bytes(bytes),
            None => Err(self.unrepresentable()),
        }
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
        assert_eq!(resident.entries(100, 32), Err(ResidentExhausted));
        assert_eq!(resident.exceeded_requirement(), Some(25_600));
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
        // Handing back what was never held is no limit of the allowance.
        assert_eq!(resident.release(1), Err(NotHeld));
        assert_eq!(resident.exceeded_requirement(), Some(1_001));
    }

    #[test]
    fn overflowing_scratch_charge_is_rejected_even_at_maximum_ceiling() {
        let mut resident = ResidentAllowance::new(u64::MAX);
        resident.bytes(1).unwrap();
        assert_eq!(resident.transient(u64::MAX), Err(ResidentExhausted));
        assert_eq!(resident.used(), 1);
        // A size past what a count of bytes can say exhausts any allowance.
        let mut bounded = ResidentAllowance::new(1 << 40);
        assert_eq!(bounded.entries(usize::MAX, 32), Err(ResidentExhausted));
        assert_eq!(bounded.exceeded_requirement(), Some(u64::MAX));
        assert_eq!(bounded.used(), 0);
    }

    #[test]
    fn a_trace_slot_the_allowance_cannot_hold_is_its_bound_not_an_entry_limit() {
        let mut trace = crate::integrity_ingress::RecoveryIntegrityIngressTrace::default();
        let mut none = ResidentAllowance::new(0);
        let unreserved = trace.owned_heap_bytes();
        assert!(matches!(
            none.trace_slots(&mut trace, 1),
            Err(ResidentTraceDenial::ResidentBoundExceeded)
        ));
        assert!(none.exceeded_requirement().is_some());
        // Refused before the backing it had no room for was reserved.
        assert_eq!(trace.owned_heap_bytes(), unreserved);
        let mut ample = ResidentAllowance::new(1 << 20);
        ample.trace_slots(&mut trace, 1).unwrap();
        assert_eq!(Some(ample.used()), trace.owned_heap_bytes());
    }
}
