//! Completed postcheckpoint V3 releases followed by an ordinary selected tip.
//! No pending descriptor or synthetic terminal fate is manufactured here.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, ReleaseCheckpointNoReleaseV1,
};
use worth_store_physical_integrity::{VerifiedCheckpointFacts, VerifiedCheckpointStream};

use super::{
    no_release_custody::selected_checkpoint_marker, PhysicalSourceSelection,
    VerifiedOrderedPendingWalReleaseBatch, VerifiedOrderedRootEdge, VerifiedOrderedRootHistory,
    VerifiedSelectedReleaseHeadCustodyV2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderedHistoricalReleaseCustodyDenial {
    Checkpoint,
    SelectedRoot,
    ReleaseRoster,
    Bound,
}

#[derive(Debug)]
pub struct VerifiedOrderedHistoricalReleaseCustody {
    checkpoint: VerifiedCheckpointFacts,
    base: OrderedHistoricalCheckpointBase,
    selected_root: DurablePhysicalRootManifest,
    selected_root_frame_sha256: [u8; 32],
    selected_free_space_frame_sha256: [u8; 32],
    history: Arc<VerifiedOrderedRootHistory>,
    batches: Box<[VerifiedOrderedPendingWalReleaseBatch]>,
}

#[derive(Debug)]
enum OrderedHistoricalCheckpointBase {
    NoRelease(ReleaseCheckpointNoReleaseV1),
    ReleasedHeadV2(VerifiedSelectedReleaseHeadCustodyV2),
}

impl VerifiedOrderedHistoricalReleaseCustody {
    /// History Arc backing is counted once by the caller; checkpoint facts are inline.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let base = match &self.base {
            OrderedHistoricalCheckpointBase::NoRelease(_) => 0,
            OrderedHistoricalCheckpointBase::ReleasedHeadV2(base) => base.owned_heap_bytes()?,
        };
        let batches = u64::try_from(self.batches.len()).ok()?.checked_mul(
            u64::try_from(std::mem::size_of::<VerifiedOrderedPendingWalReleaseBatch>()).ok()?,
        )?;
        self.batches
            .iter()
            .try_fold(base.checked_add(batches)?, |sum, batch| {
                sum.checked_add(batch.owned_heap_bytes()?)
            })
    }

    pub fn admit_no_release(
        selected: &PhysicalSourceSelection,
        stream: &VerifiedCheckpointStream,
        selected_free: &DurableFreeSpaceManifestHeader,
        history: Arc<VerifiedOrderedRootHistory>,
        batches: Vec<VerifiedOrderedPendingWalReleaseBatch>,
        maximum_retained_bytes: u64,
    ) -> Result<Self, OrderedHistoricalReleaseCustodyDenial> {
        use OrderedHistoricalReleaseCustodyDenial as Denial;
        let marker =
            selected_checkpoint_marker(selected, stream).map_err(|_| Denial::Checkpoint)?;
        Self::admit(
            selected,
            selected_free,
            history,
            batches,
            OrderedHistoricalCheckpointBase::NoRelease(marker),
            maximum_retained_bytes,
        )
    }

    pub fn admit_head_v2(
        selected: &PhysicalSourceSelection,
        selected_free: &DurableFreeSpaceManifestHeader,
        base: VerifiedSelectedReleaseHeadCustodyV2,
        history: Arc<VerifiedOrderedRootHistory>,
        batches: Vec<VerifiedOrderedPendingWalReleaseBatch>,
        maximum_retained_bytes: u64,
    ) -> Result<Self, OrderedHistoricalReleaseCustodyDenial> {
        Self::admit(
            selected,
            selected_free,
            history,
            batches,
            OrderedHistoricalCheckpointBase::ReleasedHeadV2(base),
            maximum_retained_bytes,
        )
    }

    fn admit(
        selected: &PhysicalSourceSelection,
        selected_free: &DurableFreeSpaceManifestHeader,
        history: Arc<VerifiedOrderedRootHistory>,
        batches: Vec<VerifiedOrderedPendingWalReleaseBatch>,
        base: OrderedHistoricalCheckpointBase,
        maximum_retained_bytes: u64,
    ) -> Result<Self, OrderedHistoricalReleaseCustodyDenial> {
        use OrderedHistoricalReleaseCustodyDenial as Denial;
        let checkpoint = selected.checkpoint().ok_or(Denial::Checkpoint)?;
        let root = selected.root().selected().manifest();
        let format = selected.root().selected().selector().format();
        let root_sha: [u8; 32] = Sha256::digest(root.encode(format)).into();
        let free_sha: [u8; 32] = Sha256::digest(selected_free.encode(format)).into();
        let base_matches = match &base {
            OrderedHistoricalCheckpointBase::NoRelease(marker) => {
                marker.checkpoint() == checkpoint.checkpoint().source().identity()
                    && marker.root_sha256() == checkpoint.source_root_frame_sha256()
            }
            OrderedHistoricalCheckpointBase::ReleasedHeadV2(custody) => {
                custody.checkpoint().source().identity()
                    == checkpoint.checkpoint().source().identity()
                    && custody.source_root_sha256() == checkpoint.source_root_frame_sha256()
                    && custody.selected_root() == root
            }
        };
        if !base_matches
            || history.checkpoint_root_frame_sha256() != checkpoint.source_root_frame_sha256()
        {
            return Err(Denial::Checkpoint);
        }
        if root_sha != history.selected_root_frame_sha256()
            || free_sha != history.selected_free_space_frame_sha256()
            || !history
                .selected_topology()
                .matches_headers(root, selected_free, format)
        {
            return Err(Denial::SelectedRoot);
        }
        let mut release_ordinal = 0usize;
        let mut retained_bytes = history.peak_scratch_bytes();
        for (edge_index, edge) in history.edges().iter().enumerate() {
            let VerifiedOrderedRootEdge::Released(release) = edge else {
                continue;
            };
            let batch = batches.get(release_ordinal).ok_or(Denial::ReleaseRoster)?;
            if batch.edge_index() != edge_index
                || batch.descriptor_frame().record() != release.descriptor_record()
                || batch.descriptor_frame().payload_sha256() != release.descriptor_frame_sha256()
                || batch.descriptor().custody().request().idempotency() != release.operation()
                || batch.raw_fate() != release.fate()
                || batch.wal_fate().lsn_start() != release.lsn().start().get()
                || batch.wal_fate().lsn_end_exclusive() != release.lsn().end_exclusive().get()
            {
                return Err(Denial::ReleaseRoster);
            }
            retained_bytes = retained_bytes
                .checked_add(batch.retained_bytes())
                .ok_or(Denial::Bound)?;
            release_ordinal += 1;
        }
        if release_ordinal == 0
            || release_ordinal != batches.len()
            || retained_bytes > maximum_retained_bytes
        {
            return Err(Denial::Bound);
        }
        Ok(Self {
            checkpoint: *checkpoint.checkpoint(),
            base,
            selected_root: root.clone(),
            selected_root_frame_sha256: root_sha,
            selected_free_space_frame_sha256: free_sha,
            history,
            batches: batches.into_boxed_slice(),
        })
    }

    pub fn checkpoint(&self) -> &VerifiedCheckpointFacts {
        &self.checkpoint
    }
    pub const fn marker(&self) -> Option<ReleaseCheckpointNoReleaseV1> {
        match self.base {
            OrderedHistoricalCheckpointBase::NoRelease(marker) => Some(marker),
            OrderedHistoricalCheckpointBase::ReleasedHeadV2(_) => None,
        }
    }
    pub fn selected_head_v2(&self) -> Option<&VerifiedSelectedReleaseHeadCustodyV2> {
        match &self.base {
            OrderedHistoricalCheckpointBase::NoRelease(_) => None,
            OrderedHistoricalCheckpointBase::ReleasedHeadV2(base) => Some(base),
        }
    }
    pub const fn selected_root(&self) -> &DurablePhysicalRootManifest {
        &self.selected_root
    }
    pub const fn selected_root_frame_sha256(&self) -> [u8; 32] {
        self.selected_root_frame_sha256
    }
    pub const fn selected_free_space_frame_sha256(&self) -> [u8; 32] {
        self.selected_free_space_frame_sha256
    }
    pub fn history(&self) -> &VerifiedOrderedRootHistory {
        &self.history
    }
    pub fn released_batches(&self) -> &[VerifiedOrderedPendingWalReleaseBatch] {
        &self.batches
    }
}
