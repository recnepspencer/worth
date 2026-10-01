//! The two new roster backings share one local admission. Ordered replay
//! attachments already belong to the caller and are not allocated here.

use worth_store_physical_format::ReleaseCustodyHeadEntryV1;

use crate::VerifiedOrderedReleasedHeadReplayV14;

use super::EffectiveReleaseHeadDenial;

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
    if source_requested
        .checked_add(result_requested)
        .is_none_or(|bytes| bytes > maximum_additional_resident_bytes)
    {
        return Err(EffectiveReleaseHeadDenial::ResidentBoundExceeded {
            required: source_requested
                .checked_add(result_requested)
                .unwrap_or(u64::MAX),
            admitted: maximum_additional_resident_bytes,
        });
    }
    let mut source = Vec::new();
    source
        .try_reserve_exact(source_count)
        .map_err(|_| EffectiveReleaseHeadDenial::BoundExceeded)?;
    let source_allocated = (source.capacity() as u64)
        .checked_mul(entry_bytes)
        .ok_or(EffectiveReleaseHeadDenial::BoundExceeded)?;
    if source_allocated
        .checked_add(result_requested)
        .is_none_or(|bytes| bytes > maximum_additional_resident_bytes)
    {
        return Err(EffectiveReleaseHeadDenial::ResidentBoundExceeded {
            required: source_allocated
                .checked_add(result_requested)
                .unwrap_or(u64::MAX),
            admitted: maximum_additional_resident_bytes,
        });
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(result_capacity)
        .map_err(|_| EffectiveReleaseHeadDenial::BoundExceeded)?;
    let result_allocated = (result.capacity() as u64)
        .checked_mul(entry_bytes)
        .ok_or(EffectiveReleaseHeadDenial::BoundExceeded)?;
    if source_allocated
        .checked_add(result_allocated)
        .is_none_or(|bytes| bytes > maximum_additional_resident_bytes)
    {
        return Err(EffectiveReleaseHeadDenial::ResidentBoundExceeded {
            required: source_allocated
                .checked_add(result_allocated)
                .unwrap_or(u64::MAX),
            admitted: maximum_additional_resident_bytes,
        });
    }
    Ok((source, result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_and_result_backing_share_one_additional_ceiling() {
        let width = std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64;
        let required = width * 3;
        assert_eq!(
            reserve_rosters(1, 2, required - 1).unwrap_err(),
            EffectiveReleaseHeadDenial::ResidentBoundExceeded {
                required,
                admitted: required - 1,
            }
        );
        let (source, result) = reserve_rosters(1, 2, required).unwrap();
        assert!(source.capacity() >= 1 && result.capacity() >= 2);
        assert_eq!(
            roster_backing_bytes(&source, &result),
            Some((source.capacity() as u64 + result.capacity() as u64) * width)
        );
    }
}
