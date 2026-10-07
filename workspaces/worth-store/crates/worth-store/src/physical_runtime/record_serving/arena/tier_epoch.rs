use super::{ArenaAllocationDenial, SharedArenaAllocationOwner};

/// Fences every arena claim while a root-only tier-namespace transition is
/// planned. Once WAL or namespace effects may have escaped, dropping the
/// carrier deliberately leaves the owner quarantined until fresh reopen.
pub(in crate::physical_runtime::record_serving) struct ArenaTierEpochFence {
    owner: SharedArenaAllocationOwner,
    epoch: u64,
    state: FenceState,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FenceState {
    PreEffect,
    EffectMayExist,
    Settled,
}

impl ArenaTierEpochFence {
    pub(in crate::physical_runtime::record_serving) fn begin(
        owner: &SharedArenaAllocationOwner,
        published_next_arena: u64,
    ) -> Result<Self, ArenaAllocationDenial> {
        let epoch = owner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .begin_tier_epoch_activation(published_next_arena)?;
        Ok(Self {
            owner: owner.clone(),
            epoch,
            state: FenceState::PreEffect,
        })
    }

    #[cfg(test)]
    pub(in crate::physical_runtime::record_serving) const fn epoch(&self) -> u64 {
        self.epoch
    }

    pub(in crate::physical_runtime::record_serving) fn mark_effect_may_exist(&mut self) {
        self.state = FenceState::EffectMayExist;
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn prove_no_effect(&mut self) {
        self.state = FenceState::PreEffect;
    }

    pub(in crate::physical_runtime::record_serving) fn complete(
        mut self,
    ) -> Result<(), ArenaAllocationDenial> {
        self.state = FenceState::EffectMayExist;
        self.owner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .complete_tier_epoch_activation(self.epoch)?;
        self.state = FenceState::Settled;
        Ok(())
    }
}

impl Drop for ArenaTierEpochFence {
    fn drop(&mut self) {
        if self.state == FenceState::PreEffect {
            self.owner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .abort_tier_epoch_activation_before_effect();
        }
    }
}
