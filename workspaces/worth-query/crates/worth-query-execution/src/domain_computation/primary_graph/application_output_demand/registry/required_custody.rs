//! Shared, refundable custody beneath the registry's required-retained limit.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::{DemandRegistryState, WorthQueryOutputDemandRegistry};

mod caller_chain;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// Positive reservations are serialized by registry admission. The final Arc
/// owner refunds atomically, including when it outlives its demand record.
pub(in crate::domain_computation::primary_graph) struct RequiredOutputCustodyCapacity {
    retained_bytes: Arc<AtomicUsize>,
    bytes: usize,
}

#[cfg(test)]
pub(super) fn test_capacity(bytes: usize) -> RequiredOutputCustodyCapacity {
    RequiredOutputCustodyCapacity {
        retained_bytes: Arc::new(AtomicUsize::new(bytes)),
        bytes,
    }
}

impl Drop for RequiredOutputCustodyCapacity {
    fn drop(&mut self) {
        let prior = self.retained_bytes.fetch_sub(self.bytes, Ordering::AcqRel);
        assert!(prior >= self.bytes, "required custody has one final owner");
    }
}

impl DemandRegistryState {
    pub(super) fn has_required_capacity(&self, indexed_bytes: usize) -> bool {
        indexed_bytes
            .checked_add(self.required_custody_retained_bytes.load(Ordering::Acquire))
            .is_some_and(|total| total <= self.required_budget_bytes)
    }

    pub(super) fn reserve_required_custody_capacity(
        &self,
        bytes: usize,
    ) -> Result<RequiredOutputCustodyCapacity, WorthQueryOutputDemandDenial> {
        let retained = self.required_custody_retained_bytes.load(Ordering::Acquire);
        let required = retained.checked_add(bytes).ok_or_else(capacity_denial)?;
        if self
            .required_reserved_bytes
            .checked_add(required)
            .is_none_or(|total| total > self.required_budget_bytes)
        {
            return Err(full_custody_denial(bytes, self.required_budget_bytes));
        }
        self.required_custody_retained_bytes
            .fetch_add(bytes, Ordering::AcqRel);
        Ok(RequiredOutputCustodyCapacity {
            retained_bytes: Arc::clone(&self.required_custody_retained_bytes),
            bytes,
        })
    }
}

impl WorthQueryOutputDemandRegistry {
    pub(super) fn reserve_required_custody_capacity(
        &self,
        bytes: usize,
    ) -> Result<RequiredOutputCustodyCapacity, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.reserve_required_custody_capacity(bytes)
    }

    pub(in crate::domain_computation::primary_graph) fn reserve_settlement_custody_capacity(
        &self,
        bytes: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RequiredOutputCustodyCapacity, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        let bytes_u64 = u64::try_from(bytes).map_err(|_| capacity_denial())?;
        admission
            .admit_read_scratch(bytes_u64)
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    work_denial()
                }
                _ => capacity_denial(),
            })?;
        self.reclaim_cached_rows(bytes, admission)?;
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.reserve_required_custody_capacity(bytes)
    }

    /// Prepay a caller-owned required continuation before its successor can
    /// register or execute. `peak_bytes` includes any old Vec backing that
    /// coexists while a replacement buffer is prepared; only `retained_bytes`
    /// remains charged to the final-owner ticket after installation.
    pub(in crate::domain_computation::primary_graph) fn reserve_required_continuation_capacity(
        &self,
        retained_bytes: usize,
        peak_bytes: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RequiredOutputCustodyCapacity, WorthQueryOutputDemandDenial> {
        if peak_bytes < retained_bytes {
            return Err(capacity_denial());
        }
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        admission
            .admit_read_scratch(u64::try_from(peak_bytes).map_err(|_| capacity_denial())?)
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    work_denial()
                }
                _ => capacity_denial(),
            })?;
        self.reclaim_cached_rows(peak_bytes, admission)?;
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.reserve_required_custody_capacity(retained_bytes)
    }
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "required output custody exceeds request work",
    )
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "required output custody exceeds retained capacity",
    )
}

/// Custody frees as other required rows release, so a reservation that fits
/// the budget on its own may succeed on a later advance. The caller's advance,
/// or its start, decides whether one can: see `caller_chain`.
pub(super) fn full_custody_denial(bytes: usize, budget: usize) -> WorthQueryOutputDemandDenial {
    let denial = capacity_denial();
    if bytes > budget {
        return denial;
    }
    denial.with_recovery_posture(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable,
    )
}
