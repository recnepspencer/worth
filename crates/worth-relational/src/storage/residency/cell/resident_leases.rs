//! Borrow existing physical custody without loading or bypassing root admission.
use super::*;

impl RelationalRegionResidency {
    pub(in crate::storage::residency) fn lease_checkpoint_contents(
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
        match &*state {
            ResidencyState::Head { region } | ResidencyState::Full(region) => {
                self.leases
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                        count.checked_add(1)
                    })
                    .map_err(|_| self.failure(Cause::MemoryExhausted))?;
                Ok(Some(RelationalResidencyLease::accepted(
                    self.clone(),
                    region.clone(),
                )))
            }
            _ => Ok(None),
        }
    }

    /// A raw cold-backed image still needs its root's explicit serving admission.
    pub(crate) fn lease_resident_head(
        self: &Arc<Self>,
    ) -> Result<Option<RelationalResidencyLease<Head>>, RelationalStorageUnavailable> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.check_damage()?;
        match &*state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region)
                if self.backing.is_none() =>
            {
                self.leases
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                        count.checked_add(1)
                    })
                    .map_err(|_| self.failure(Cause::MemoryExhausted))?;
                Ok(Some(RelationalResidencyLease::accepted(
                    self.clone(),
                    region.clone(),
                )))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn lease_resident_full(
        self: &Arc<Self>,
    ) -> Result<Option<RelationalResidencyLease<Full>>, RelationalStorageUnavailable> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.check_damage()?;
        match &*state {
            ResidencyState::Full(region) if self.backing.is_none() => {
                self.leases
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                        count.checked_add(1)
                    })
                    .map_err(|_| self.failure(Cause::MemoryExhausted))?;
                Ok(Some(RelationalResidencyLease::accepted(
                    self.clone(),
                    region.clone(),
                )))
            }
            _ => Ok(None),
        }
    }
}
