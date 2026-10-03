//! Allocation-first accounting under one original Store recovery admission.
//! Each ownership boundary supplies its actual live backing and may only reduce the ceiling.

use crate::physical_runtime::PhysicalRecoveryAllocationAdmission;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoveryRejoinResidentDenial {
    MissingResidentAdmission,
    OperationAllocation(crate::physical_runtime::PhysicalScopedAllocationFailure),
    SizeOverflow {
        admitted: u64,
    },
    BudgetExceeded {
        required: u64,
        admitted: u64,
    },
    Allocation {
        requested: u64,
        cause: std::collections::TryReserveError,
    },
    AllocatorExceededReservation {
        requested: u64,
        actual: u64,
    },
}

use PhysicalRecoveryRejoinResidentDenial as StoreRejoinResidentDenial;

pub(in crate::physical_runtime) struct StoreRejoinResidentLedger {
    used: u64,
    maximum: u64,
    peak: u64,
    failed: Option<StoreRejoinResidentDenial>,
}

impl StoreRejoinResidentLedger {
    /// The pending head observer's existing local allowance, clamped to the
    /// original admission. This does not account for surrounding C8/Store
    /// backing; V2 must consume its carried coordination basis instead.
    #[cfg(feature = "recovery-runtime-owner")]
    pub(super) fn pending_head_walk_window(
        allocation: PhysicalRecoveryAllocationAdmission,
        maximum_resident_bytes: u64,
    ) -> Result<Self, StoreRejoinResidentDenial> {
        Self::new(allocation, 0, maximum_resident_bytes)
    }

    #[cfg(test)]
    pub(super) fn for_test(
        allocation: PhysicalRecoveryAllocationAdmission,
        already_live: u64,
        maximum_additional: u64,
    ) -> Result<Self, StoreRejoinResidentDenial> {
        Self::new(allocation, already_live, maximum_additional)
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_coordination(
        coordination: &mut crate::physical_runtime::PhysicalRecoveryCoordination,
    ) -> Result<Self, StoreRejoinResidentDenial> {
        let (allocation, already_live) = coordination
            .take_rejoin_resident_basis()
            .ok_or(StoreRejoinResidentDenial::MissingResidentAdmission)?;
        Self::new(allocation, already_live, u64::MAX)
    }

    pub(in crate::physical_runtime) fn from_retained_with_limit(
        allocation: PhysicalRecoveryAllocationAdmission,
        already_live: u64,
        maximum_total: u64,
    ) -> Result<Self, StoreRejoinResidentDenial> {
        let additional = maximum_total.checked_sub(already_live).ok_or(
            StoreRejoinResidentDenial::BudgetExceeded {
                required: already_live,
                admitted: maximum_total,
            },
        )?;
        Self::new(allocation, already_live, additional)
    }

    fn new(
        allocation: PhysicalRecoveryAllocationAdmission,
        already_live: u64,
        maximum_additional: u64,
    ) -> Result<Self, StoreRejoinResidentDenial> {
        let remaining = allocation.byte_limit().checked_sub(already_live).ok_or(
            StoreRejoinResidentDenial::BudgetExceeded {
                required: already_live,
                admitted: allocation.byte_limit(),
            },
        )?;
        Ok(Self {
            used: already_live,
            maximum: already_live + remaining.min(maximum_additional),
            peak: already_live,
            failed: None,
        })
    }

    #[cfg(any(test, feature = "recovery-runtime-owner"))]
    pub(in crate::physical_runtime) const fn remaining(&self) -> u64 {
        if self.failed.is_some() {
            0
        } else {
            self.maximum.saturating_sub(self.used)
        }
    }

    pub(in crate::physical_runtime) const fn used(&self) -> u64 {
        self.used
    }

    pub(in crate::physical_runtime) const fn peak(&self) -> u64 {
        self.peak
    }

    pub(in crate::physical_runtime) fn transient(
        &mut self,
        bytes: u64,
    ) -> Result<(), StoreRejoinResidentDenial> {
        if let Some(denial) = &self.failed {
            return Err(denial.clone());
        }
        let required = self
            .used
            .checked_add(bytes)
            .ok_or_else(|| self.overflow())?;
        if required > self.maximum {
            return Err(StoreRejoinResidentDenial::BudgetExceeded {
                required,
                admitted: self.maximum,
            });
        }
        self.peak = self.peak.max(required);
        Ok(())
    }

    pub(in crate::physical_runtime) fn retain(
        &mut self,
        bytes: u64,
    ) -> Result<(), StoreRejoinResidentDenial> {
        self.transient(bytes)?;
        self.used += bytes;
        Ok(())
    }

    #[cfg(any(test, feature = "recovery-runtime-owner"))]
    pub(in crate::physical_runtime) fn release(&mut self, bytes: u64) {
        self.used = self
            .used
            .checked_sub(bytes)
            .expect("release only dropped charged backing");
    }

    pub(in crate::physical_runtime) fn reserve_vec<T>(
        &mut self,
        count: usize,
    ) -> Result<Vec<T>, StoreRejoinResidentDenial> {
        let requested = self.slot_bytes::<T>(count)?;
        self.transient(requested)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|cause| StoreRejoinResidentDenial::Allocation { requested, cause })?;
        let actual = self.vector_bytes(&values)?;
        self.transient(actual)?;
        self.retain(actual)?;
        Ok(values)
    }

    #[cfg(any(test, feature = "recovery-runtime-owner"))]
    pub(in crate::physical_runtime) fn reserve_bytes(
        &mut self,
        count: usize,
    ) -> Result<Vec<u8>, StoreRejoinResidentDenial> {
        let mut bytes = self.reserve_vec(count)?;
        bytes.resize(count, 0);
        Ok(bytes)
    }

    /// The new allocation is admitted while the old vector is still live.
    pub(in crate::physical_runtime) fn grow_vec<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), StoreRejoinResidentDenial> {
        self.transient(0)?;
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or_else(|| self.overflow())?;
        self.grow_to_capacity(values, needed)
    }

    /// Streaming frame rosters need amortized growth, not one reallocation per
    /// admitted frame. A larger capacity is still admitted with the old backing
    /// live; an insufficient budget denies instead of silently changing cost.
    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn grow_vec_geometrically<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), StoreRejoinResidentDenial> {
        self.transient(0)?;
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or_else(|| self.overflow())?;
        if needed <= values.capacity() {
            return Ok(());
        }
        let doubled_capacity = values
            .capacity()
            .checked_mul(2)
            .ok_or_else(|| self.overflow())?;
        self.grow_to_capacity(values, needed.max(doubled_capacity))
    }

    fn grow_to_capacity<T>(
        &mut self,
        values: &mut Vec<T>,
        needed: usize,
    ) -> Result<(), StoreRejoinResidentDenial> {
        if needed <= values.capacity() {
            return Ok(());
        }
        let requested = self.slot_bytes::<T>(needed)?;
        self.transient(requested)?;
        let previous = self.vector_bytes(values)?;
        values
            .try_reserve_exact(needed - values.len())
            .map_err(|cause| StoreRejoinResidentDenial::Allocation { requested, cause })?;
        // try_reserve_exact may provide extra capacity. Reconcile the actual
        // new backing while the previous backing is still charged; the final
        // capacity delta alone does not describe the reallocation peak.
        self.reconcile_growth(previous, self.vector_bytes(values)?)
    }

    fn reconcile_growth(
        &mut self,
        previous: u64,
        actual: u64,
    ) -> Result<(), StoreRejoinResidentDenial> {
        if let Err(denial) = self.transient(actual) {
            // Reservation has already changed the caller-owned vector. This
            // ledger may no longer authorize work using its previous charge.
            self.used = self.used.saturating_sub(previous).saturating_add(actual);
            self.peak = self.peak.max(self.used.saturating_add(previous));
            self.failed = Some(denial.clone());
            return Err(denial);
        }
        self.retain(
            actual
                .checked_sub(previous)
                .ok_or_else(|| self.overflow())?,
        )
    }

    pub(in crate::physical_runtime) fn vector_bytes<T>(
        &self,
        values: &Vec<T>,
    ) -> Result<u64, StoreRejoinResidentDenial> {
        self.slot_bytes::<T>(values.capacity())
    }

    fn slot_bytes<T>(&self, count: usize) -> Result<u64, StoreRejoinResidentDenial> {
        u64::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(std::mem::size_of::<T>() as u64))
            .ok_or_else(|| self.overflow())
    }

    fn overflow(&self) -> StoreRejoinResidentDenial {
        StoreRejoinResidentDenial::SizeOverflow {
            admitted: self.maximum,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    fn admission(bytes: u64) -> PhysicalRecoveryAllocationAdmission {
        // Identity is irrelevant to these local allocation arithmetic tests;
        // this published-record meaning grants no media or custody authority.
        let identity = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([1; 16]).unwrap(),
        )
        .published_identity();
        PhysicalRecoveryAllocationAdmission::new(identity, bytes)
    }

    #[test]
    fn retained_state_and_reducing_ceiling_precede_allocation() {
        assert!(matches!(
            StoreRejoinResidentLedger::from_retained_with_limit(admission(100), 80, 79),
            Err(StoreRejoinResidentDenial::BudgetExceeded {
                required: 80,
                admitted: 79
            })
        ));
        let original_ceiling =
            StoreRejoinResidentLedger::from_retained_with_limit(admission(95), 80, 110).unwrap();
        assert_eq!(original_ceiling.remaining(), 15);
        let mut window =
            StoreRejoinResidentLedger::from_retained_with_limit(admission(100), 80, 90).unwrap();
        assert_eq!((window.used(), window.remaining()), (80, 10));
        assert_eq!(
            window.reserve_bytes(11),
            Err(StoreRejoinResidentDenial::BudgetExceeded {
                required: 91,
                admitted: 90
            })
        );
        assert_eq!((window.used(), window.peak()), (80, 80));
        let bytes = window.reserve_bytes(10).unwrap();
        assert_eq!((window.used(), window.peak()), (90, 90));
        let charged = window.vector_bytes(&bytes).unwrap();
        drop(bytes);
        window.release(charged);
        assert_eq!(
            (window.used(), window.peak(), window.remaining()),
            (80, 90, 10)
        );
    }

    #[test]
    fn vector_growth_accounts_for_overlapping_backing_and_preserves_rejected_values() {
        let mut window = StoreRejoinResidentLedger::new(admission(32), 20, u64::MAX).unwrap();
        let mut values = window.reserve_bytes(4).unwrap();
        window.grow_vec(&mut values, 4).unwrap();
        values.extend_from_slice(&[1; 4]);
        assert_eq!((window.used(), window.peak()), (28, 32));
        assert_eq!(
            window.grow_vec(&mut values, 1),
            Err(StoreRejoinResidentDenial::BudgetExceeded {
                required: 37,
                admitted: 32
            })
        );
        assert_eq!(
            (values.as_slice(), values.capacity()),
            (&[0, 0, 0, 0, 1, 1, 1, 1][..], 8)
        );
    }

    #[test]
    fn allocator_capacity_failure_keeps_the_underlying_cause() {
        let mut window = StoreRejoinResidentLedger::new(admission(u64::MAX), 0, u64::MAX).unwrap();
        let StoreRejoinResidentDenial::Allocation { requested, cause } =
            window.reserve_bytes(usize::MAX).unwrap_err()
        else {
            panic!("allocator rejection must not become declared-budget exhaustion")
        };
        assert_eq!(requested, usize::MAX as u64);
        assert_eq!(
            cause,
            Vec::<u8>::new().try_reserve_exact(usize::MAX).unwrap_err()
        );
        assert_eq!(window.used(), 0);
    }
}
