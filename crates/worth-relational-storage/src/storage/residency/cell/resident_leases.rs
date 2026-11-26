//! Borrow existing physical custody without loading or bypassing root admission.
use super::*;
use crate::storage::residency::working_set::ResidencyLevelKind;

impl RelationalRegionResidency {
    #[doc(hidden)]
    pub fn has_checkpoint_contents_for_test(
        self: &Arc<Self>,
    ) -> Result<bool, RelationalStorageUnavailable> {
        self.lease_checkpoint_contents()
            .map(|contents| contents.is_some())
    }

    pub(in crate::internal_port::storage::residency) fn lease_checkpoint_contents(
        self: &Arc<Self>,
    ) -> Result<
        Option<RelationalResidencyLease<crate::storage::residency::lease::CheckpointContents>>,
        RelationalStorageUnavailable,
    > {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.check_damage()?;
        match &**state {
            ResidencyState::Head { region } | ResidencyState::Full(region) => {
                self.count_lease()?;

                Ok(Some(RelationalResidencyLease::accepted(
                    self.clone(),
                    region.clone(),
                )))
            }
            _ => Ok(None),
        }
    }

    /// A raw cold-backed image still needs its root's explicit serving admission.
    pub fn lease_resident_head(
        self: &Arc<Self>,
    ) -> Result<Option<RelationalResidencyLease<Head>>, RelationalStorageUnavailable> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.check_damage()?;
        match &**state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region)
                if self.backing.is_none() =>
            {
                self.count_lease()?;

                Ok(Some(RelationalResidencyLease::accepted(
                    self.clone(),
                    region.clone(),
                )))
            }
            _ => Ok(None),
        }
    }

    pub fn lease_resident_full(
        self: &Arc<Self>,
    ) -> Result<Option<RelationalResidencyLease<Full>>, RelationalStorageUnavailable> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.check_damage()?;
        match &**state {
            ResidencyState::Full(region) if self.backing.is_none() => {
                self.count_lease()?;

                Ok(Some(RelationalResidencyLease::accepted(
                    self.clone(),
                    region.clone(),
                )))
            }
            _ => Ok(None),
        }
    }

    fn count_lease(&self) -> Result<(), RelationalStorageUnavailable> {
        let mut count = self.leases.load(Ordering::Acquire);
        loop {
            let updated = count
                .checked_add(1)
                .ok_or_else(|| self.failure(Cause::MemoryExhausted))?;
            match self.leases.compare_exchange_weak(
                count,
                updated,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(()),
                Err(reloaded) => count = reloaded,
            }
        }
    }

    /// Borrow the image this cell already serves, with no I/O and no new
    /// working-set charge: a new operation's substitute for root retention
    /// (Store M4 D1). `None` for a cold, damaged or foreign-proved cell.
    pub(in crate::internal_port::storage::residency) fn lease_served(
        self: &Arc<Self>,
        authority: Option<&super::super::RelationalRegionKindAuthority>,
    ) -> Option<(SharedResidentRegion, ResidencyLevelKind)> {
        if let Some(authority) = authority {
            authority.require_cell(self).ok()?;
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.check_damage().ok()?;
        let served = match &**state {
            ResidencyState::Head { region } => (region.clone(), ResidencyLevelKind::Head),
            ResidencyState::Full(region) => (region.clone(), ResidencyLevelKind::Full),
            _ => return None,
        };
        self.count_lease().ok()?;
        drop(state);
        self.mark_referenced();
        Some(served)
    }
}
