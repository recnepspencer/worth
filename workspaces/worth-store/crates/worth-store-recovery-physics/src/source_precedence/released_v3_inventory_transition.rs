//! One canonical, bounded V3 source-to-result inventory predicate shared by
//! C.8 and Store. A transcript of two valid trees alone is not a legal drop.

use super::VerifiedReleasedDirectoryReplacement;
use crate::VerifiedSelectedReleaseHeadReplayV14;

#[path = "released_v3_inventory_transition/delta.rs"]
mod delta;
use delta::{next_extent, root_semantics_match, routes_match, validate_arenas};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    ExtentArenaId, ExtentArenaRange, FreeSpaceKey, PersistedRecordIdentity,
    PhysicalInventoryTranscriptBuilderV1, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration, RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleasedV3InventoryTransitionDenial {
    Allocation {
        requested_bytes: u64,
        cause: std::collections::TryReserveError,
    },
    BoundExceeded,
    InvalidSource,
    InvalidResult,
    InvalidDelta,
}

#[derive(Clone, Copy)]
pub struct ReleasedInventoryView<'a> {
    pub root: &'a DurablePhysicalRootManifest,
    pub free: &'a DurableFreeSpaceManifestHeader,
    pub routes: &'a [CurrentPhysicalRecordPlacement],
    pub segments: &'a [RecordSegmentPageManifestEntry],
    pub free_entries: &'a [RecordFreeSpaceManifestEntry],
}

impl<'a> ReleasedInventoryView<'a> {
    pub fn new(
        root: &'a DurablePhysicalRootManifest,
        free: &'a DurableFreeSpaceManifestHeader,
        routes: &'a [CurrentPhysicalRecordPlacement],
        segments: &'a [RecordSegmentPageManifestEntry],
        free_entries: &'a [RecordFreeSpaceManifestEntry],
    ) -> Self {
        Self {
            root,
            free,
            routes,
            segments,
            free_entries,
        }
    }
}

/// This owned token can be minted only by checking an exact V3 inventory
/// transition. Store must still independently rewalk actual media and call
/// the same constructor; caller-supplied views have no inherent authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedReleasedV3InventoryTransition {
    source: PhysicalInventoryTranscriptV1,
    result: PhysicalInventoryTranscriptV1,
    scratch_bytes: u64,
    projected: Box<[CurrentPhysicalRecordPlacement]>,
    directory_replacement: Option<VerifiedReleasedDirectoryReplacement>,
}

impl VerifiedReleasedV3InventoryTransition {
    /// Additional construction peak, excluding the borrowed source/result inputs.
    /// Headers, sorted subtraction ranges, and retained projected-copy conversion
    /// occupy separate windows; each is admitted before its first reservation.
    pub fn maximum_construction_heap_bytes(
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        projected: &[CurrentPhysicalRecordPlacement],
        maximum_entries: u64,
    ) -> Option<u64> {
        if source.routes.len() as u64 > maximum_entries
            || result.routes.len() as u64 > maximum_entries
            || source.free_entries.len() as u64 > maximum_entries
            || result.free_entries.len() as u64 > maximum_entries
            || projected.len() as u64 > maximum_entries
        {
            return None;
        }
        let headers = |view: ReleasedInventoryView<'_>| {
            (view.root.encoded_frame_bytes() as u64)
                .checked_add(view.free.encoded_frame_bytes() as u64)
        };
        let ranges =
            (projected.len() as u64).checked_mul(std::mem::size_of::<ExtentArenaRange>() as u64)?;
        let retained = (projected.len() as u64)
            .checked_mul(std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64)?
            .checked_mul(2)?;
        Some(
            headers(source)?
                .max(headers(result)?)
                .max(ranges)
                .max(retained),
        )
    }

    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.projected.len())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<CurrentPhysicalRecordPlacement>()).ok()?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        dropped: &[PersistedRecordIdentity],
        projected: &[CurrentPhysicalRecordPlacement],
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        maximum_scratch_bytes: u64,
        directory_replacement: Option<&VerifiedReleasedDirectoryReplacement>,
    ) -> Result<Self, ReleasedV3InventoryTransitionDenial> {
        Self::admit_inner(
            source,
            result,
            dropped,
            projected,
            None,
            directory_replacement,
            format,
            maximum_entries,
            maximum_scratch_bytes,
        )
    }

    /// The head-tree fields may change only under an exact, C.9-admitted
    /// WAL effect whose source path was re-read from selected media.
    #[allow(clippy::too_many_arguments)]
    pub fn admit_with_head_replay(
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        dropped: &[PersistedRecordIdentity],
        projected: &[CurrentPhysicalRecordPlacement],
        head_replay: &VerifiedSelectedReleaseHeadReplayV14,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        maximum_scratch_bytes: u64,
        directory_replacement: Option<&VerifiedReleasedDirectoryReplacement>,
    ) -> Result<Self, ReleasedV3InventoryTransitionDenial> {
        Self::admit_inner(
            source,
            result,
            dropped,
            projected,
            Some(head_replay),
            directory_replacement,
            format,
            maximum_entries,
            maximum_scratch_bytes,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn admit_inner(
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        dropped: &[PersistedRecordIdentity],
        projected: &[CurrentPhysicalRecordPlacement],
        head_replay: Option<&VerifiedSelectedReleaseHeadReplayV14>,
        directory_replacement: Option<&VerifiedReleasedDirectoryReplacement>,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        maximum_scratch_bytes: u64,
    ) -> Result<Self, ReleasedV3InventoryTransitionDenial> {
        use ReleasedV3InventoryTransitionDenial as Denial;
        if maximum_entries == 0 || maximum_scratch_bytes == 0 {
            return Err(Denial::BoundExceeded);
        }
        let scratch_bytes =
            Self::maximum_construction_heap_bytes(source, result, projected, maximum_entries)
                .filter(|bytes| *bytes <= maximum_scratch_bytes)
                .ok_or(Denial::BoundExceeded)?;
        let source_topology =
            transcript_reserved(source, format, maximum_entries, maximum_scratch_bytes)?;
        let result_topology =
            transcript_reserved(result, format, maximum_entries, maximum_scratch_bytes).map_err(
                |denial| match denial {
                    Denial::InvalidSource => Denial::InvalidResult,
                    other => other,
                },
            )?;
        if !root_semantics_match(
            source,
            result,
            dropped,
            projected,
            head_replay,
            directory_replacement,
        ) || dropped.is_empty()
            || projected.is_empty()
            || dropped.len() as u64 > maximum_entries
            || projected.len() as u64 > maximum_entries
            || dropped.windows(2).any(|pair| pair[0] >= pair[1])
            || projected
                .windows(2)
                .any(|pair| pair[0].record() >= pair[1].record())
            || projected.iter().any(|route| {
                !matches!(route, CurrentPhysicalRecordPlacement::Extent(_))
                    || source
                        .routes
                        .binary_search_by_key(&route.record(), |item| item.record())
                        .is_ok()
            })
            || dropped.iter().any(|record| {
                source
                    .routes
                    .binary_search_by_key(record, |item| item.record())
                    .is_err()
            })
            || source.segments != result.segments
        {
            return Err(Denial::InvalidDelta);
        }
        if !routes_match(source.routes, result.routes, dropped, projected)
            || result.free.next_segment() != source.free.next_segment()
            || result.free.next_page() != source.free.next_page()
            || result.free.next_extent()
                != next_extent(source.free, result.routes).ok_or(Denial::InvalidDelta)?
        {
            return Err(Denial::InvalidDelta);
        }
        let next_arena = validate_arenas(
            source,
            result,
            projected,
            maximum_entries,
            maximum_scratch_bytes,
        )?;
        if result.free.next_arena() != next_arena
            || result.free.tree_identity() != source.free.tree_identity()
            || result.free.node_capacity() != source.free.node_capacity()
            || result.free.segment_page_capacity() != source.free.segment_page_capacity()
            || result.free.arena_capacity() != source.free.arena_capacity()
            || result.free.arena_alignment() != source.free.arena_alignment()
            || result.free.tier_epoch_start() != source.free.tier_epoch_start()
        {
            return Err(Denial::InvalidDelta);
        }
        let mut retained_projected =
            reserve::<CurrentPhysicalRecordPlacement>(projected.len(), 0, maximum_scratch_bytes)?;
        retained_projected.extend_from_slice(projected);
        let conversion_peak = ((retained_projected.capacity() + retained_projected.len()) as u64)
            .checked_mul(std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64)
            .ok_or(Denial::BoundExceeded)?;
        if conversion_peak > maximum_scratch_bytes {
            return Err(Denial::BoundExceeded);
        }
        Ok(Self {
            source: source_topology,
            result: result_topology,
            scratch_bytes: scratch_bytes.max(conversion_peak),
            projected: retained_projected.into_boxed_slice(),
            directory_replacement: directory_replacement
                .map(|proof| proof.bind_result_root(result.root)),
        })
    }

    pub const fn source_topology(&self) -> PhysicalInventoryTranscriptV1 {
        self.source
    }
    pub const fn result_topology(&self) -> PhysicalInventoryTranscriptV1 {
        self.result
    }
    pub const fn scratch_bytes(&self) -> u64 {
        self.scratch_bytes
    }
    pub fn projected(&self) -> &[CurrentPhysicalRecordPlacement] {
        &self.projected
    }
    pub const fn directory_replacement(&self) -> Option<&VerifiedReleasedDirectoryReplacement> {
        self.directory_replacement.as_ref()
    }
}

pub(crate) fn transcript(
    view: ReleasedInventoryView<'_>,
    format: PhysicalRecordFormatDeclaration,
    limit: u64,
) -> Result<PhysicalInventoryTranscriptV1, ReleasedV3InventoryTransitionDenial> {
    use ReleasedV3InventoryTransitionDenial as Denial;
    let mut builder =
        PhysicalInventoryTranscriptBuilderV1::new(view.root, view.free, format, limit)
            .map_err(|_| Denial::InvalidSource)?;
    for route in view.routes {
        builder
            .include_route(*route)
            .map_err(|_| Denial::InvalidSource)?;
    }
    for segment in view.segments {
        builder
            .include_segment(*segment)
            .map_err(|_| Denial::InvalidSource)?;
    }
    for free in view.free_entries {
        builder
            .include_free(*free)
            .map_err(|_| Denial::InvalidSource)?;
    }
    builder.finish().map_err(|_| Denial::InvalidSource)
}

pub(super) fn reserve<T>(
    count: usize,
    retained: u64,
    maximum: u64,
) -> Result<Vec<T>, ReleasedV3InventoryTransitionDenial> {
    use ReleasedV3InventoryTransitionDenial as Denial;
    let requested_bytes = (count as u64)
        .checked_mul(std::mem::size_of::<T>() as u64)
        .ok_or(Denial::BoundExceeded)?;
    if retained
        .checked_add(requested_bytes)
        .is_none_or(|bytes| bytes > maximum)
    {
        return Err(Denial::BoundExceeded);
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|cause| Denial::Allocation {
            requested_bytes,
            cause,
        })?;
    let actual = (values.capacity() as u64)
        .checked_mul(std::mem::size_of::<T>() as u64)
        .ok_or(Denial::BoundExceeded)?;
    if retained
        .checked_add(actual)
        .is_none_or(|bytes| bytes > maximum)
    {
        return Err(Denial::BoundExceeded);
    }
    Ok(values)
}

fn transcript_reserved(
    view: ReleasedInventoryView<'_>,
    format: PhysicalRecordFormatDeclaration,
    limit: u64,
    maximum: u64,
) -> Result<PhysicalInventoryTranscriptV1, ReleasedV3InventoryTransitionDenial> {
    use ReleasedV3InventoryTransitionDenial as Denial;
    let free = reserve::<u8>(view.free.encoded_frame_bytes(), 0, maximum)?;
    let root = reserve::<u8>(
        view.root.encoded_frame_bytes(),
        free.capacity() as u64,
        maximum,
    )?;
    let mut builder = PhysicalInventoryTranscriptBuilderV1::new_in_reserved(
        view.root, view.free, format, limit, root, free,
    )
    .map_err(|_| Denial::InvalidSource)?;
    for route in view.routes {
        builder
            .include_route(*route)
            .map_err(|_| Denial::InvalidSource)?;
    }
    for segment in view.segments {
        builder
            .include_segment(*segment)
            .map_err(|_| Denial::InvalidSource)?;
    }
    for free in view.free_entries {
        builder
            .include_free(*free)
            .map_err(|_| Denial::InvalidSource)?;
    }
    builder.finish().map_err(|_| Denial::InvalidSource)
}

#[cfg(test)]
#[path = "released_v3_inventory_transition/directory_tests.rs"]
mod directory_tests;
#[cfg(test)]
#[path = "released_v3_inventory_transition/tests.rs"]
mod tests;
