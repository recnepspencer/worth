use worth_store_physical_format::{BlobManifestResidueCleanup, BlobManifestResidueCleanupPhaseV1};
use worth_store_wal::WalLsnRange;

use super::StoreRecoveryBindingSampleDenial;

pub(super) type Observation = (WalLsnRange, BlobManifestResidueCleanup, bool);

/// Frames are the sole live collector. Pairing sorts this buffer in place,
/// then restores LSN order; no second map or uncharged node allocation exists.
pub(super) struct ManifestCleanupObservations {
    frames: Vec<Observation>,
    peak_bytes: u64,
}

impl ManifestCleanupObservations {
    pub(super) fn prepare(
        frames: Vec<Observation>,
        memory_limit: u64,
    ) -> Result<Self, StoreRecoveryBindingSampleDenial> {
        if !frames.is_empty() {
            return Err(StoreRecoveryBindingSampleDenial::InvalidWalMember);
        }
        let peak_bytes = u64::try_from(frames.capacity())
            .ok()
            .and_then(|count| count.checked_mul(std::mem::size_of::<Observation>() as u64))
            .ok_or(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit)?;
        if peak_bytes > memory_limit {
            return Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit);
        }
        Ok(Self { frames, peak_bytes })
    }

    pub(super) fn observe(
        &mut self,
        range: WalLsnRange,
        cleanup: BlobManifestResidueCleanup,
        store: [u8; 16],
        maximum: u64,
    ) -> Result<(), StoreRecoveryBindingSampleDenial> {
        if cleanup.store() != store {
            return Err(StoreRecoveryBindingSampleDenial::InvalidWalMember);
        }
        if self.frames.len() as u64 >= maximum {
            return Err(StoreRecoveryBindingSampleDenial::OperationBindingLimit);
        }
        if self
            .frames
            .last()
            .is_some_and(|(previous, _, _)| range.start().get() < previous.end_exclusive().get())
        {
            return Err(StoreRecoveryBindingSampleDenial::InvalidWalMember);
        }
        if self.frames.len() == self.frames.capacity() {
            return Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit);
        }
        self.frames.push((range, cleanup, false));
        Ok(())
    }

    pub(super) fn finish(
        mut self,
    ) -> Result<(Vec<Observation>, u64), StoreRecoveryBindingSampleDenial> {
        self.frames.sort_unstable_by_key(|(range, cleanup, _)| {
            (cleanup.reclaim_attempt(), range.start().get())
        });
        let mut position = 0;
        while position < self.frames.len() {
            let attempt = self.frames[position].1.reclaim_attempt();
            let first = self.frames[position];
            let next = self
                .frames
                .get(position + 1)
                .copied()
                .filter(|(_, value, _)| value.reclaim_attempt() == attempt);
            if self
                .frames
                .get(position + 2)
                .is_some_and(|(_, value, _)| value.reclaim_attempt() == attempt)
            {
                return Err(StoreRecoveryBindingSampleDenial::InvalidWalMember);
            }
            match (first.1.phase(), next) {
                (BlobManifestResidueCleanupPhaseV1::Intent, Some(completion))
                    if completion.1.phase() == BlobManifestResidueCleanupPhaseV1::Completed
                        && first.1.completed() == completion.1
                        && first.0.end_exclusive().get() <= completion.0.start().get() =>
                {
                    self.frames[position].2 = true;
                    self.frames[position + 1].2 = true;
                    position += 2;
                }
                (_, Some(_)) => return Err(StoreRecoveryBindingSampleDenial::InvalidWalMember),
                (_, None) => position += 1,
            }
        }
        self.frames
            .sort_unstable_by_key(|(range, _, _)| range.start().get());
        Ok((self.frames, self.peak_bytes))
    }
}

#[cfg(test)]
mod tests;
