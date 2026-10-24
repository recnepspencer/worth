//! How much one batch of this door may carry, fixed before any effect.
//!
//! A batch is admitted by the record access policy (records and bytes per
//! append), by the extent arena, which holds every extent-backed record of a
//! batch as an in-flight reservation beside its free ranges within the
//! placement policy's arena range budget, and by the WAL, which places a
//! batch's whole group (data frames and root projection) in one WAL segment.
//! Each refuses an oversized batch before any effect. The door plans batches
//! within every bound with headroom: half the arena range budget for extents,
//! leaving the rest for free ranges, and half a WAL segment for record bytes,
//! leaving the rest for framing and the root projection. A batch the runtime
//! still refuses (an arena more fragmented than the headroom) is a typed
//! `MemberWrite` refusal that leaves the anchor where it was.

use crate::physical_runtime::record_serving::arena_range_budget;
use crate::physical_runtime::{AdmittedRecordPlacementPolicy, ServingPhysicalRuntime};

/// The share of one WAL segment a batch's record bytes may fill.
const WAL_SEGMENT_SHARE_DIVISOR: u64 = 2;
/// The share of the arena range budget a batch's extents may reserve.
const ARENA_RANGE_SHARE_DIVISOR: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BatchCapacity {
    pub(super) records: usize,
    pub(super) bytes: u64,
    pub(super) extents: usize,
    /// A record of at least this many bytes is extent-backed.
    pub(super) extent_threshold: u64,
}

impl BatchCapacity {
    pub(super) fn of(
        runtime: &ServingPhysicalRuntime,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Self {
        let access = runtime.record_access_policy();
        let segment = runtime
            .durability_observation()
            .wal_policy()
            .segment_byte_limit()
            .get()
            .get();
        let ranges = arena_range_budget(u64::from(placement.arena_index_bytes().get()));
        Self {
            records: access.append_record_limit().get() as usize,
            bytes: u64::from(access.append_byte_limit().get())
                .min(u64::from(u32::MAX))
                .min(segment / WAL_SEGMENT_SHARE_DIVISOR),
            extents: (ranges / ARENA_RANGE_SHARE_DIVISOR).max(1),
            extent_threshold: u64::from(placement.extent_threshold().get()),
        }
    }

    /// Whether a record of `length` bytes reserves an arena range.
    pub(super) const fn is_extent(&self, length: u64) -> bool {
        length >= self.extent_threshold
    }
}
