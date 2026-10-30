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
                {
                    let update_counter = &self.leases;
                    let mut count = update_counter.load(Ordering::Acquire);
                    loop {
                        let Some(updated_value) = count.checked_add(1) else {
                            break Err(count);
                        };
                        match update_counter.compare_exchange_weak(
                            count,
                            updated_value,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        ) {
                            Ok(previous) => break Ok(previous),
                            Err(reloaded_value) => count = reloaded_value,
                        }
                    }
                }
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
                {
                    let update_counter = &self.leases;
                    let mut count = update_counter.load(Ordering::Acquire);
                    loop {
                        let Some(updated_value) = count.checked_add(1) else {
                            break Err(count);
                        };
                        match update_counter.compare_exchange_weak(
                            count,
                            updated_value,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        ) {
                            Ok(previous) => break Ok(previous),
                            Err(reloaded_value) => count = reloaded_value,
                        }
                    }
                }
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
                {
                    let update_counter = &self.leases;
                    let mut count = update_counter.load(Ordering::Acquire);
                    loop {
                        let Some(updated_value) = count.checked_add(1) else {
                            break Err(count);
                        };
                        match update_counter.compare_exchange_weak(
                            count,
                            updated_value,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        ) {
                            Ok(previous) => break Ok(previous),
                            Err(reloaded_value) => count = reloaded_value,
                        }
                    }
                }
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
