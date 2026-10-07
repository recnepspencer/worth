//! The V3 transition over inventories whose segments are not yet laid out as
//! slices. The check lays them out itself, under the one scratch ceiling it
//! checks with, so a limit counts the segments and the check together.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, RecordFreeSpaceManifestEntry,
    RecordSegmentPageManifestEntry,
};

use super::transcripts::reserve;
use super::{
    ReleasedInventoryView, ReleasedV3InventoryTransitionDenial, RootHistoryAllowance,
    VerifiedReleasedDirectoryReplacement, VerifiedReleasedV3InventoryTransition,
};
use crate::VerifiedSelectedReleaseHeadReplayV14;

/// One side of a transition whose segments are still an iterator.
#[derive(Clone, Copy)]
pub struct ReleasedInventoryParts<'a, Segments> {
    pub root: &'a DurablePhysicalRootManifest,
    pub free: &'a DurableFreeSpaceManifestHeader,
    pub routes: &'a [CurrentPhysicalRecordPlacement],
    pub segments: Segments,
    pub free_entries: &'a [RecordFreeSpaceManifestEntry],
}

type Denial = ReleasedV3InventoryTransitionDenial;
const SEGMENT_WIDTH: u64 = std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64;

impl VerifiedReleasedV3InventoryTransition {
    /// Lays out both sides' segments, then checks the transition with what
    /// they leave of `maximum_scratch_bytes`. Returns the transition and the
    /// scratch the laid-out segments hold beside it. A limit names both
    /// together, against `maximum_scratch_bytes`.
    #[allow(clippy::too_many_arguments)]
    pub fn admit_laid_out<S, R>(
        source: ReleasedInventoryParts<'_, S>,
        result: ReleasedInventoryParts<'_, R>,
        dropped: &[PersistedRecordIdentity],
        projected: &[CurrentPhysicalRecordPlacement],
        head_replay: Option<&VerifiedSelectedReleaseHeadReplayV14>,
        directory_replacement: Option<&VerifiedReleasedDirectoryReplacement>,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        maximum_scratch_bytes: u64,
    ) -> Result<(Self, u64), Denial>
    where
        S: ExactSizeIterator<Item = RecordSegmentPageManifestEntry>,
        R: ExactSizeIterator<Item = RecordSegmentPageManifestEntry>,
    {
        // Both sides are refused together before either is laid out.
        let both = (source.segments.len() as u64)
            .checked_add(result.segments.len() as u64)
            .and_then(|count| count.checked_mul(SEGMENT_WIDTH))
            .ok_or(Denial::SizeOverflow)?;
        RootHistoryAllowance::scratch_bytes(maximum_scratch_bytes)
            .admit(both)
            .map_err(Denial::BoundExceeded)?;
        let source_segments = lay_out(source.segments, 0, maximum_scratch_bytes)?;
        // `reserve` admitted each capacity within the ceiling.
        let source_held = source_segments.capacity() as u64 * SEGMENT_WIDTH;
        let result_segments = lay_out(result.segments, source_held, maximum_scratch_bytes)?;
        let held = source_held + result_segments.capacity() as u64 * SEGMENT_WIDTH;
        let source_view = ReleasedInventoryView::new(
            source.root,
            source.free,
            source.routes,
            &source_segments,
            source.free_entries,
        );
        let result_view = ReleasedInventoryView::new(
            result.root,
            result.free,
            result.routes,
            &result_segments,
            result.free_entries,
        );
        let transition = Self::admit_inner(
            source_view,
            result_view,
            dropped,
            projected,
            head_replay,
            directory_replacement,
            format,
            maximum_entries,
            maximum_scratch_bytes - held,
        )
        .map_err(|denial| match denial {
            // The check had what the segments left; they hold the rest.
            Denial::BoundExceeded(narrower) => RootHistoryAllowance::held_beside(narrower, held)
                .map_or(Denial::SizeOverflow, Denial::BoundExceeded),
            other => other,
        })?;
        Ok((transition, held))
    }
}

fn lay_out(
    segments: impl ExactSizeIterator<Item = RecordSegmentPageManifestEntry>,
    retained: u64,
    maximum: u64,
) -> Result<Vec<RecordSegmentPageManifestEntry>, Denial> {
    let mut laid_out = reserve(segments.len(), retained, maximum)?;
    laid_out.extend(segments);
    Ok(laid_out)
}

#[cfg(test)]
#[path = "laid_out_tests.rs"]
mod tests;
