use std::collections::BTreeSet;
use std::ops::Range;

use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalSegmentId, RecordSegmentPageManifestEntry,
};

use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::access::manifest_routing::ManifestDiscoveryCounterSnapshot;
use crate::physical_runtime::record_serving::access::segment_membership::SegmentMembershipReader;
use crate::physical_runtime::record_serving::planning::inline_plan_failure::manifest_lookup_failure;
use crate::physical_runtime::record_serving::{RecordAppendDenial, RecordAppendError};

/// What the current root still reads from a rewrite's source generation.
///
/// Append generations are compact and a sparse span rewrite leaves holes, so
/// neither frame position nor file length says which frames are live. Only
/// the current membership entries that name the generation do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RewriteSpanLiveness {
    /// Every frame in the span is live and the root reads other frames of the
    /// generation too; the file must stay after the rewrite.
    LiveWithRetainedFrames,
    /// The span covers every live frame, so the rewrite leaves the generation
    /// unreachable from the resulting root.
    LiveAndDisplacesSource,
    /// Some frame in the span is not a live frame of this generation.
    NotLive,
}

impl RewriteSpanLiveness {
    pub(super) fn classify(
        entries: &[RecordSegmentPageManifestEntry],
        generation: u64,
        span: Range<u32>,
    ) -> Self {
        let live = entries
            .iter()
            .filter(|entry| entry.data_generation() == generation)
            .map(|entry| entry.frame_index())
            .collect::<BTreeSet<_>>();
        if span.is_empty() || !span.clone().all(|frame| live.contains(&frame)) {
            return Self::NotLive;
        }
        if live.iter().all(|frame| span.contains(frame)) {
            Self::LiveAndDisplacesSource
        } else {
            Self::LiveWithRetainedFrames
        }
    }

    /// Admits the span or reports the typed denial before any effect.
    pub(super) fn displaces_source(self) -> Result<bool, RecordAppendError> {
        match self {
            Self::LiveWithRetainedFrames => Ok(false),
            Self::LiveAndDisplacesSource => Ok(true),
            Self::NotLive => Err(RecordAppendError::Denied(
                RecordAppendDenial::RewriteSpanNotLive,
            )),
        }
    }
}

impl RecordPublicationDirector {
    /// Classifies the rewrite span against the root's membership of `segment`.
    ///
    /// Reads only the membership blocks that hold `segment`.
    pub(super) fn rewrite_source_liveness(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        root: &DurablePhysicalRootManifest,
        segment: PhysicalSegmentId,
        generation: u64,
        span: Range<u32>,
    ) -> Result<RewriteSpanLiveness, RecordAppendError> {
        let mut discovery = ManifestDiscoveryCounterSnapshot::default();
        let entries = SegmentMembershipReader::serving(
            self.residency.clone(),
            self.format,
            self.access,
            root.clone(),
        )
        .segment_entries(allocation, segment, &mut discovery)
        .map_err(manifest_lookup_failure)?;
        Ok(RewriteSpanLiveness::classify(&entries, generation, span))
    }
}
