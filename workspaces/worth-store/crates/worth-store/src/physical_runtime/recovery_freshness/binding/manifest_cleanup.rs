use worth_store_physical_format::{BlobManifestResidueCleanup, BlobManifestResidueCleanupPhaseV1};
use worth_store_wal::WalLsnRange;

use super::StoreRecoveryBindingSampleDenial;

type Observation = (WalLsnRange, BlobManifestResidueCleanup, bool);

/// Frames are the sole live collector. Pairing sorts this buffer in place,
/// then restores LSN order; no second map or uncharged node allocation exists.
pub(super) struct ManifestCleanupObservations {
    frames: Vec<Observation>,
    memory_limit: u64,
    peak_bytes: u64,
}

impl ManifestCleanupObservations {
    pub(super) fn new(memory_limit: u64) -> Self {
        Self {
            frames: Vec::new(),
            memory_limit,
            peak_bytes: 0,
        }
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
        self.reserve_one(maximum)?;
        self.frames.push((range, cleanup, false));
        Ok(())
    }

    fn reserve_one(&mut self, maximum: u64) -> Result<(), StoreRecoveryBindingSampleDenial> {
        if self.frames.len() < self.frames.capacity() {
            return Ok(());
        }
        let old = self.frames.capacity();
        let maximum = usize::try_from(maximum).unwrap_or(usize::MAX);
        let next = old.max(1).saturating_mul(2).min(maximum);
        let unit = std::mem::size_of::<Observation>() as u64;
        // A growth may retain the old buffer while allocating the new one.
        // Boxing at finish can likewise coexist with the final Vec buffer.
        let transient = (old as u64)
            .checked_add(next as u64)
            .and_then(|count| count.checked_mul(unit))
            .ok_or(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit)?;
        let boxed = (next as u64)
            .checked_mul(2)
            .and_then(|count| count.checked_mul(unit))
            .ok_or(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit)?;
        let admitted_peak = transient.max(boxed);
        if next <= old || admitted_peak > self.memory_limit {
            return Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit);
        }
        self.frames
            .try_reserve_exact(next - self.frames.len())
            .map_err(|_| StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit)?;
        // reserve_exact does not deliberately over-allocate. If the allocator
        // grants extra capacity, that capacity must also fit the live bound.
        let actual = (self.frames.capacity() as u64)
            .checked_mul(2)
            .and_then(|count| count.checked_mul(unit))
            .ok_or(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit)?;
        if actual > self.memory_limit {
            return Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit);
        }
        self.peak_bytes = self.peak_bytes.max(admitted_peak).max(actual);
        Ok(())
    }

    pub(super) fn finish(
        mut self,
    ) -> Result<(Box<[Observation]>, u64), StoreRecoveryBindingSampleDenial> {
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
        Ok((self.frames.into_boxed_slice(), self.peak_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::PersistedRecordIdentity;
    use worth_store_wal::{LogSequenceNumber, WalLsnRange};

    fn interval(start: u64) -> WalLsnRange {
        WalLsnRange::new(
            LogSequenceNumber::new(start),
            LogSequenceNumber::new(start + 1),
        )
        .unwrap()
    }

    fn intent(sha: u8) -> BlobManifestResidueCleanup {
        worth_store_physical_format::BlobManifestResidueCleanupV1::intent(
            [1; 16],
            [2; 16],
            PersistedRecordIdentity::new([3; 16], 4).unwrap(),
            [sha; 32],
            [5; 32],
            [6; 32],
            [7; 32],
            8,
            9,
            [10; 32],
            11,
            12,
        )
        .unwrap()
        .into()
    }

    #[test]
    fn observed_intent_and_completion_are_ordered_and_exact() {
        let mut sample = ManifestCleanupObservations::new(16_384);
        let first_intent = intent(4);
        sample
            .observe(interval(20), first_intent, [1; 16], 2)
            .unwrap();
        sample
            .observe(interval(22), first_intent.completed(), [1; 16], 2)
            .unwrap();
        let (frames, peak) = sample.finish().unwrap();
        assert_eq!(frames.len(), 2);
        assert!(frames[0].2 && frames[1].2);
        assert!(peak >= 2 * std::mem::size_of::<Observation>() as u64);
    }

    #[test]
    fn covered_intent_may_leave_only_a_completion_in_the_tail() {
        let mut sample = ManifestCleanupObservations::new(16_384);
        sample
            .observe(interval(22), intent(4).completed(), [1; 16], 1)
            .unwrap();
        let (frames, _) = sample.finish().unwrap();
        assert_eq!(frames.len(), 1);
        assert!(!frames[0].2);
    }

    #[test]
    fn duplicates_conflicts_foreign_store_and_reversed_order_are_denied() {
        let mut sample = ManifestCleanupObservations::new(16_384);
        let first_intent = intent(4);
        assert_eq!(
            sample.observe(interval(20), first_intent, [9; 16], 2),
            Err(StoreRecoveryBindingSampleDenial::InvalidWalMember)
        );
        sample
            .observe(interval(20), first_intent, [1; 16], 2)
            .unwrap();
        sample
            .observe(interval(21), intent(5).completed(), [1; 16], 2)
            .unwrap();
        assert_eq!(
            sample.finish().err(),
            Some(StoreRecoveryBindingSampleDenial::InvalidWalMember)
        );
        let mut reversed = ManifestCleanupObservations::new(16_384);
        reversed
            .observe(interval(20), first_intent.completed(), [1; 16], 2)
            .unwrap();
        reversed
            .observe(interval(21), first_intent, [1; 16], 2)
            .unwrap();
        assert_eq!(
            reversed.finish().err(),
            Some(StoreRecoveryBindingSampleDenial::InvalidWalMember)
        );
    }

    #[test]
    fn tight_memory_limit_denies_before_any_frame_allocation() {
        let mut sample =
            ManifestCleanupObservations::new(2 * std::mem::size_of::<Observation>() as u64 - 1);
        assert_eq!(
            sample.observe(interval(20), intent(4), [1; 16], 1),
            Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit)
        );
        assert_eq!(sample.frames.capacity(), 0);
    }
}
