//! A bound cell's working-set custody: binding, non-blocking eviction and
//! observation. Admission and charging of loads live in `working_set_charge`.
use super::*;
use crate::storage::residency::working_set::evictor::EvictionAttempt;
use crate::storage::residency::working_set::{ResidencyPoolKind, WorkingSetAccountant};
use std::sync::atomic::AtomicBool;

#[derive(Debug)]
pub(in crate::internal_port::storage::residency) struct WorkingSetBinding {
    pub(super) accountant: Arc<WorkingSetAccountant>,
    pub(super) pool: ResidencyPoolKind,
    /// Clock reference bit: set by every demand, cleared by one sweep.
    pub(super) referenced: AtomicBool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingSetBindingRefusal {
    /// Only cells backed by this home's verified durable members can be
    /// rewarmed after eviction; an unbacked cell would lose committed data.
    Unbacked,
    AlreadyBound,
}

/// Point-in-time cell observation; grants no admission or eviction authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellWorkingSetObservation {
    pub pool: Option<ResidencyPoolKind>,
    pub leases: usize,
    pub resident: bool,
    pub raw_head_cached: bool,
    pub raw_full_cached: bool,
    /// Working-set bytes charged to the region the cell state serves.
    pub state_bytes: u64,
    /// Charged bytes held by raw caches beyond the served region.
    pub raw_cache_bytes: u64,
}

fn charged(region: &ResidentRegion) -> u64 {
    region.working_set.get().map_or(0, |charge| charge.bytes())
}

fn cached_region(cache: &RegionLoadCache) -> Option<SharedResidentRegion> {
    match &*cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
    {
        Some(Ok(region)) => Some(region.clone()),
        Some(Err(_)) | None => None,
    }
}

impl RelationalRegionResidency {
    /// Bind a cold-backed cell to one home's working-set pool. Its later loads
    /// reserve before they publish and it becomes an eviction candidate.
    pub fn bind_working_set(
        self: &Arc<Self>,
        accountant: &Arc<WorkingSetAccountant>,
        pool: ResidencyPoolKind,
    ) -> Result<(), WorkingSetBindingRefusal> {
        if self.backing.is_none() {
            return Err(WorkingSetBindingRefusal::Unbacked);
        }
        self.working_set
            .set(WorkingSetBinding {
                accountant: accountant.clone(),
                pool,
                referenced: AtomicBool::new(true),
            })
            .map_err(|_| WorkingSetBindingRefusal::AlreadyBound)?;
        accountant.enroll(Self::downgrade(self));
        Ok(())
    }

    /// Evict only an unborrowed, undamaged, resident cell whose members this
    /// home still holds. Every gate is taken with `try_lock`; a contended cell
    /// is skipped, never waited for.
    pub(in crate::internal_port::storage::residency) fn try_evict(
        &self,
        accountant: &Arc<WorkingSetAccountant>,
        pool: ResidencyPoolKind,
    ) -> EvictionAttempt {
        let Some(binding) = self.working_set.get() else {
            return EvictionAttempt::Ineligible;
        };
        if !Arc::ptr_eq(&binding.accountant, accountant) || binding.pool != pool {
            return EvictionAttempt::Ineligible;
        }
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::WouldBlock) => return EvictionAttempt::Busy,
            Err(std::sync::TryLockError::Poisoned(_)) => return EvictionAttempt::Ineligible,
        };
        let members_held = self.backing.is_some()
            && state
                .physical
                .as_ref()
                .and_then(|physical| physical.source())
                .is_some();
        if self.damage.get().is_some()
            || !members_held
            || !matches!(
                **state,
                ResidencyState::Head { .. } | ResidencyState::Full(_)
            )
        {
            return EvictionAttempt::Ineligible;
        }
        // Counts rise only under this gate or from an already-counted lease,
        // so zero here cannot become one until the gate is released.
        if self.leases.load(Ordering::Acquire) != 0
            || binding.referenced.swap(false, Ordering::AcqRel)
        {
            return EvictionAttempt::Borrowed;
        }
        let (Ok(mut raw_head), Ok(mut raw_full)) =
            (self.raw_head.try_lock(), self.raw_full.try_lock())
        else {
            return EvictionAttempt::Busy;
        };
        let served = std::mem::replace(&mut **state, ResidencyState::Cold);
        // Proven-damage refusals stay sticky; only resident images are dropped.
        let head = raw_head.take_if(|cached| cached.is_ok());
        let full = raw_full.take_if(|cached| cached.is_ok());
        drop((raw_full, raw_head, state));
        drop((served, head, full));
        EvictionAttempt::Evicted
    }

    pub(super) fn mark_referenced(&self) {
        if let Some(binding) = self.working_set.get() {
            binding.referenced.store(true, Ordering::Release);
        }
    }

    /// The level this cell currently serves, without admission or I/O.
    pub fn resident_level(
        &self,
    ) -> Option<crate::storage::residency::working_set::ResidencyLevelKind> {
        use crate::storage::residency::working_set::ResidencyLevelKind as Level;
        match &**self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
        {
            ResidencyState::Head { .. } => Some(Level::Head),
            ResidencyState::Full(_) => Some(Level::Full),
            ResidencyState::Cold | ResidencyState::Damaged { .. } => None,
        }
    }

    pub fn working_set_observation(&self) -> CellWorkingSetObservation {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let served = match &**state {
            ResidencyState::Head { region } | ResidencyState::Full(region) => Some(region.clone()),
            ResidencyState::Cold | ResidencyState::Damaged { .. } => None,
        };
        let raw_head = cached_region(&self.raw_head);
        let raw_full = cached_region(&self.raw_full);
        drop(state);
        let served_ptr = served
            .as_ref()
            .map(|region| &**region as *const ResidentRegion);
        let raw_cache_bytes = [&raw_head, &raw_full]
            .into_iter()
            .flatten()
            .filter(|region| Some(&***region as *const ResidentRegion) != served_ptr)
            .map(|region| charged(region))
            .sum();
        CellWorkingSetObservation {
            pool: self.working_set.get().map(|binding| binding.pool),
            leases: self.leases.load(Ordering::Acquire),
            resident: served.is_some(),
            raw_head_cached: raw_head.is_some(),
            raw_full_cached: raw_full.is_some(),
            state_bytes: served.as_deref().map_or(0, charged),
            raw_cache_bytes,
        }
    }
}
