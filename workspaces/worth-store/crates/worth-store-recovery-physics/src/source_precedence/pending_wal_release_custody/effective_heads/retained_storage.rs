//! The two new roster backings share one local admission. Ordered replay
//! attachments already belong to the caller and are not allocated here.

use worth_store_physical_format::ReleaseCustodyHeadEntryV1;

use crate::VerifiedOrderedReleasedHeadReplayV14;

use super::EffectiveReleaseHeadDenial;
use crate::source_precedence::PhysicsAllowance;

#[derive(Debug)]
pub(in crate::source_precedence::pending_wal_release_custody) struct PreparedEffectiveHeadRosterV14
{
    pub(super) checkpoint_source_heads: Vec<ReleaseCustodyHeadEntryV1>,
    pub(super) effective_heads: Vec<ReleaseCustodyHeadEntryV1>,
    pub(super) ordered_replays: Vec<(usize, VerifiedOrderedReleasedHeadReplayV14)>,
}

impl PreparedEffectiveHeadRosterV14 {
    pub(in crate::source_precedence::pending_wal_release_custody) fn owned_heap_bytes(
        &self,
    ) -> Option<u64> {
        let attachments = u64::try_from(self.ordered_replays.capacity())
            .ok()?
            .checked_mul(
                u64::try_from(std::mem::size_of::<(
                    usize,
                    VerifiedOrderedReleasedHeadReplayV14,
                )>())
                .ok()?,
            )?;
        self.ordered_replays.iter().try_fold(
            self.new_roster_backing_bytes()?.checked_add(attachments)?,
            |sum, (_, replay)| sum.checked_add(replay.owned_heap_bytes()?),
        )
    }

    pub(super) fn new_roster_backing_bytes(&self) -> Option<u64> {
        roster_backing_bytes(&self.checkpoint_source_heads, &self.effective_heads)
    }
}

pub(super) fn roster_backing_bytes(
    source: &Vec<ReleaseCustodyHeadEntryV1>,
    result: &Vec<ReleaseCustodyHeadEntryV1>,
) -> Option<u64> {
    let width = std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64;
    (source.capacity() as u64)
        .checked_mul(width)?
        .checked_add((result.capacity() as u64).checked_mul(width)?)
}

pub(super) fn reserve_rosters(
    source_count: usize,
    result_capacity: usize,
    maximum_additional_resident_bytes: u64,
) -> Result<
    (
        Vec<ReleaseCustodyHeadEntryV1>,
        Vec<ReleaseCustodyHeadEntryV1>,
    ),
    EffectiveReleaseHeadDenial,
> {
    let entry_bytes = std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64;
    let source_requested = (source_count as u64)
        .checked_mul(entry_bytes)
        .ok_or(EffectiveReleaseHeadDenial::BoundExceeded)?;
    let result_requested = (result_capacity as u64)
        .checked_mul(entry_bytes)
        .ok_or(EffectiveReleaseHeadDenial::BoundExceeded)?;
    let resident = PhysicsAllowance::resident_bytes(maximum_additional_resident_bytes);
    // A total past every count is no limit: the same overflow as each part.
    let sum = |source: u64, result: u64| {
        source
            .checked_add(result)
            .ok_or(EffectiveReleaseHeadDenial::BoundExceeded)
    };
    resident
        .admit(sum(source_requested, result_requested)?)
        .map_err(EffectiveReleaseHeadDenial::Limit)?;
    let mut source = Vec::new();
    source
        .try_reserve_exact(source_count)
        .map_err(|_| EffectiveReleaseHeadDenial::BoundExceeded)?;
    let source_allocated = (source.capacity() as u64)
        .checked_mul(entry_bytes)
        .ok_or(EffectiveReleaseHeadDenial::BoundExceeded)?;
    resident
        .admit(sum(source_allocated, result_requested)?)
        .map_err(EffectiveReleaseHeadDenial::Limit)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(result_capacity)
        .map_err(|_| EffectiveReleaseHeadDenial::BoundExceeded)?;
    let result_allocated = (result.capacity() as u64)
        .checked_mul(entry_bytes)
        .ok_or(EffectiveReleaseHeadDenial::BoundExceeded)?;
    resident
        .admit(sum(source_allocated, result_allocated)?)
        .map_err(EffectiveReleaseHeadDenial::Limit)?;
    Ok((source, result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_precedence::PhysicsBound;

    #[test]
    fn source_and_result_backing_share_one_additional_ceiling() {
        let width = std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64;
        let required = width * 3;
        let EffectiveReleaseHeadDenial::Limit(past) =
            reserve_rosters(1, 2, required - 1).unwrap_err()
        else {
            panic!("a roster past the ceiling is its limit");
        };
        assert_eq!(
            (past.dimension(), past.observed(), past.admitted()),
            (PhysicsBound::ResidentBytes, required, required - 1)
        );
        let (source, result) = reserve_rosters(1, 2, required).unwrap();
        assert!(source.capacity() >= 1 && result.capacity() >= 2);
        assert_eq!(
            roster_backing_bytes(&source, &result),
            Some((source.capacity() as u64 + result.capacity() as u64) * width)
        );
    }
}
