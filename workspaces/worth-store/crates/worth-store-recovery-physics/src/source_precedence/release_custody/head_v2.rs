//! Checkpoint-source V2 head roster. The selected checkpoint stream commits
//! the root reference and roster digest; C.8 still reads every rooted block.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadEntryV1,
};
use worth_store_physical_integrity::{
    walk_release_custody_head, ReleaseCustodyHeadWalkDenial, ReleaseCustodyHeadWalkLimitsV1,
    VerifiedCheckpointStream,
};

use super::{roster_v2, SelectedCustodyDenial};
use crate::PhysicalSourceSelection;

#[path = "head_v2/controls.rs"]
mod controls;
#[cfg(test)]
#[path = "head_v2/tests.rs"]
mod tests;
use controls::verify_head_controls;
pub use controls::AddressedReleaseHeadControlV2;

/// V2 selected custody is distinct from legacy tag-7 custody. A terminal
/// retirement can leave the global tip absent from the keyed head roster, so
/// this claim never infers a tip head from the V1 accumulator fields.
#[derive(Debug)]
pub struct VerifiedCheckpointReleaseHeadRosterV2 {
    checkpoint: Arc<VerifiedCheckpointStream>,
    checkpoint_source_root: DurablePhysicalRootManifest,
    source_root_sha256: [u8; 32],
    selected_root: DurablePhysicalRootManifest,
    selected_root_sha256: [u8; 32],
    batches: Vec<ReleaseCheckpointBatchV1>,
    accumulator: ReleaseCheckpointAccumulatorV2,
    selected_heads: Vec<ReleaseCustodyHeadEntryV1>,
    release_certificate_record_count: u16,
    release_certificate_encoded_bytes: u32,
    head_walk_frame_bytes: u64,
    head_walk_peak_resident_bytes: u64,
    admission_peak_resident_bytes: u64,
}

/// Control-joined selected custody. A tree/digest observation alone cannot
/// authorize a release or a Serving seal.
#[derive(Debug)]
pub struct VerifiedSelectedReleaseHeadCustodyV2 {
    roster: VerifiedCheckpointReleaseHeadRosterV2,
}

/// A failed rooted observation carries its mechanical walker or media cause;
/// neither branch is selected-custody authority.
#[derive(Debug)]
pub enum SelectedHeadRosterAdmissionDenial<ReadError> {
    Custody(SelectedCustodyDenial),
    Walk(ReleaseCustodyHeadWalkDenial<ReadError, ()>),
    Allocation {
        requested: u64,
        cause: std::collections::TryReserveError,
    },
}

impl<ReadError> From<SelectedCustodyDenial> for SelectedHeadRosterAdmissionDenial<ReadError> {
    fn from(value: SelectedCustodyDenial) -> Self {
        Self::Custody(value)
    }
}

impl VerifiedCheckpointReleaseHeadRosterV2 {
    #[allow(clippy::too_many_arguments)]
    pub fn admit_checkpoint_source<Read, ReadError>(
        selected: &PhysicalSourceSelection,
        checkpoint_source_root: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        walk_limits: ReleaseCustodyHeadWalkLimitsV1,
        maximum_head_entries: u64,
        maximum_resident_bytes: u64,
        read: Read,
    ) -> Result<Self, SelectedHeadRosterAdmissionDenial<ReadError>>
    where
        Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    {
        let denial = SelectedCustodyDenial::ReleaseBinding;
        let checkpoint = selected
            .checkpoint()
            .ok_or(SelectedCustodyDenial::MissingCheckpoint)?;
        let stream = checkpoint.checkpoint();
        let selected_candidate = selected.root().selected();
        let source = stream.source().root();
        // Root encoding holds its payload and canonical frame together.
        let root_hash_scratch = u64::from(format.page_size().bytes())
            .checked_mul(2)
            .ok_or(denial)?;
        require_resident(root_hash_scratch, maximum_resident_bytes)?;
        let source_sha: [u8; 32] = Sha256::digest(checkpoint_source_root.encode(format)).into();
        if format != selected_candidate.selector().format()
            || checkpoint_source_root.generation() != source.generation()
            || checkpoint_source_root.tree_identity() != source.tree_identity()
            || checkpoint_source_root.tree_identity()
                != selected_candidate.manifest().tree_identity()
            || source_sha != checkpoint.source_root_frame_sha256()
        {
            return Err(denial.into());
        }
        let roster = roster_v2::parse(stream, source_sha, maximum_resident_bytes)?;
        let count = roster.accumulator.head_count();
        if count > maximum_head_entries {
            return Err(denial.into());
        }
        let batch_bytes = (roster.batches.capacity() as u64)
            .checked_mul(std::mem::size_of::<ReleaseCheckpointBatchV1>() as u64)
            .ok_or(denial)?;
        let head_count = usize::try_from(count).map_err(|_| denial)?;
        let requested_heads = (head_count as u64)
            .checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)
            .ok_or(denial)?;
        let preallocation = batch_bytes.checked_add(requested_heads).ok_or(denial)?;
        require_resident(
            preallocation
                .checked_add(root_hash_scratch)
                .unwrap_or(u64::MAX),
            maximum_resident_bytes,
        )?;
        let mut heads = Vec::new();
        heads.try_reserve_exact(head_count).map_err(|cause| {
            SelectedHeadRosterAdmissionDenial::Allocation {
                requested: requested_heads,
                cause,
            }
        })?;
        let owned_heap = batch_bytes
            .checked_add(
                (heads.capacity() as u64)
                    .checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)
                    .ok_or(denial)?,
            )
            .ok_or(denial)?;
        require_resident(
            owned_heap
                .checked_add(root_hash_scratch)
                .unwrap_or(u64::MAX),
            maximum_resident_bytes,
        )?;
        let remaining = maximum_resident_bytes - owned_heap;
        if checkpoint_source_root.release_custody_head_root().is_some() {
            let minimum = ReleaseCustodyHeadWalkLimitsV1::minimum_root_resident_bytes(format)
                .ok_or(denial)?;
            require_resident(
                owned_heap.checked_add(minimum).unwrap_or(u64::MAX),
                maximum_resident_bytes,
            )?;
        }
        let walk_limits = walk_limits.with_max_resident_bytes(remaining).ok_or(
            SelectedCustodyDenial::ResidentBoundExceeded {
                required: owned_heap.saturating_add(1),
                admitted: maximum_resident_bytes,
            },
        )?;
        let walk =
            walk_release_custody_head(checkpoint_source_root, format, walk_limits, read, |entry| {
                if heads.len() as u64 >= count {
                    return Err(());
                }
                heads.push(entry);
                Ok(())
            })
            .map_err(|failure| match failure {
                ReleaseCustodyHeadWalkDenial::ResidentBoundExceeded { required, admitted } => {
                    let aggregate = owned_heap.checked_add(required).unwrap_or(u64::MAX);
                    if aggregate > maximum_resident_bytes {
                        SelectedHeadRosterAdmissionDenial::Custody(
                            SelectedCustodyDenial::ResidentBoundExceeded {
                                required: aggregate,
                                admitted: maximum_resident_bytes,
                            },
                        )
                    } else {
                        // A stricter caller-provided walker cap is not an
                        // exhaustion of this aggregate admission.
                        SelectedHeadRosterAdmissionDenial::Walk(
                            ReleaseCustodyHeadWalkDenial::ResidentBoundExceeded {
                                required,
                                admitted,
                            },
                        )
                    }
                }
                failure => SelectedHeadRosterAdmissionDenial::Walk(failure),
            })?;
        if walk.entry_count() != count
            || walk.roster_digest() != roster.accumulator.head_roster_digest()
            || heads.len() as u64 != count
        {
            return Err(denial.into());
        }
        let peak = roster
            .parse_peak_resident_bytes
            .max(root_hash_scratch)
            .max(
                owned_heap
                    .checked_add(walk.peak_resident_bytes())
                    .ok_or(denial)?,
            )
            .max(owned_heap.checked_add(root_hash_scratch).ok_or(denial)?);
        require_resident(peak, maximum_resident_bytes)?;
        let selected_root = selected_candidate.manifest().clone();
        let selected_sha = Sha256::digest(selected_root.encode(format)).into();
        Ok(Self {
            checkpoint: checkpoint.share_checkpoint(),
            checkpoint_source_root: checkpoint_source_root.clone(),
            source_root_sha256: source_sha,
            selected_root,
            selected_root_sha256: selected_sha,
            batches: roster.batches,
            accumulator: roster.accumulator,
            selected_heads: heads,
            release_certificate_record_count: roster.record_count,
            release_certificate_encoded_bytes: roster.encoded_bytes,
            head_walk_frame_bytes: walk.frame_bytes_read(),
            head_walk_peak_resident_bytes: walk.peak_resident_bytes(),
            admission_peak_resident_bytes: peak,
        })
    }

    pub fn checkpoint(&self) -> &VerifiedCheckpointStream {
        &self.checkpoint
    }
    pub const fn checkpoint_source_root(&self) -> &DurablePhysicalRootManifest {
        &self.checkpoint_source_root
    }
    pub const fn source_root_sha256(&self) -> [u8; 32] {
        self.source_root_sha256
    }
    pub const fn selected_root(&self) -> &DurablePhysicalRootManifest {
        &self.selected_root
    }
    pub const fn selected_root_sha256(&self) -> [u8; 32] {
        self.selected_root_sha256
    }
    pub const fn accumulator_v2(&self) -> ReleaseCheckpointAccumulatorV2 {
        self.accumulator
    }
    pub fn batches(&self) -> &[ReleaseCheckpointBatchV1] {
        &self.batches
    }
    pub fn selected_heads(&self) -> &[ReleaseCustodyHeadEntryV1] {
        &self.selected_heads
    }
    pub const fn release_certificate_record_count(&self) -> u16 {
        self.release_certificate_record_count
    }
    pub const fn release_certificate_encoded_bytes(&self) -> u32 {
        self.release_certificate_encoded_bytes
    }
    pub const fn head_walk_frame_bytes(&self) -> u64 {
        self.head_walk_frame_bytes
    }
    pub const fn head_walk_peak_resident_bytes(&self) -> u64 {
        self.head_walk_peak_resident_bytes
    }

    /// Only allocations owned by this claim; the checkpoint Arc and inline
    /// roots/accumulator are not newly owned heap backing.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        (self.batches.capacity() as u64)
            .checked_mul(std::mem::size_of::<ReleaseCheckpointBatchV1>() as u64)?
            .checked_add(
                (self.selected_heads.capacity() as u64)
                    .checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)?,
            )
    }

    pub const fn admission_peak_resident_bytes(&self) -> u64 {
        self.admission_peak_resident_bytes
    }

    pub fn join_controls(
        self,
        checkpoint_source_routes: &[CurrentPhysicalRecordPlacement],
        head_controls: Vec<AddressedReleaseHeadControlV2>,
    ) -> Result<VerifiedSelectedReleaseHeadCustodyV2, SelectedCustodyDenial> {
        verify_head_controls(
            &self.selected_heads,
            &self.batches,
            self.accumulator.base().prior_cumulative_dropped(),
            &head_controls,
            checkpoint_source_routes,
            self.checkpoint.source().identity().store_identity().bytes(),
        )?;
        Ok(VerifiedSelectedReleaseHeadCustodyV2 { roster: self })
    }
}

fn require_resident(required: u64, admitted: u64) -> Result<(), SelectedCustodyDenial> {
    if required > admitted {
        Err(SelectedCustodyDenial::ResidentBoundExceeded { required, admitted })
    } else {
        Ok(())
    }
}

impl VerifiedSelectedReleaseHeadCustodyV2 {
    /// Rebinds only an ordinary, authenticated publication that leaves the
    /// release-head tree untouched. V14 root changes require the separate
    /// C.9-admitted replay token and cannot enter by this method.
    pub fn rebind_published_root(
        &mut self,
        selected: &PhysicalSourceSelection,
        published: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), SelectedCustodyDenial> {
        let old = &self.roster.selected_root;
        if old != selected.root().selected().manifest()
            || format != selected.root().selected().selector().format()
            || published.tree_identity() != old.tree_identity()
            || published.tier_epoch_anchor() != old.tier_epoch_anchor()
            || published.generation() < old.generation()
            || (published.generation() == old.generation() && published != old)
            || published.release_custody_head_root() != old.release_custody_head_root()
            || published.next_release_custody_head_block() != old.next_release_custody_head_block()
            || published.generation() < self.roster.checkpoint_source_root.generation()
        {
            return Err(SelectedCustodyDenial::ReleaseBinding);
        }
        self.roster.selected_root_sha256 = Sha256::digest(published.encode(format)).into();
        self.roster.selected_root = published.clone();
        Ok(())
    }
    /// Store independently reads checkpoint-source routes and control frames
    /// and repeats the exact semantic join before installing its ledger.
    pub fn revalidate_controls(
        &self,
        checkpoint_source_routes: &[CurrentPhysicalRecordPlacement],
        catalog: &[AddressedReleaseHeadControlV2],
    ) -> Result<(), SelectedCustodyDenial> {
        verify_head_controls(
            self.selected_heads(),
            self.batches(),
            self.accumulator_v2().base().prior_cumulative_dropped(),
            catalog,
            checkpoint_source_routes,
            self.checkpoint()
                .source()
                .identity()
                .store_identity()
                .bytes(),
        )
    }
    pub fn checkpoint(&self) -> &VerifiedCheckpointStream {
        self.roster.checkpoint()
    }
    pub const fn checkpoint_source_root(&self) -> &DurablePhysicalRootManifest {
        self.roster.checkpoint_source_root()
    }
    pub const fn source_root_sha256(&self) -> [u8; 32] {
        self.roster.source_root_sha256()
    }
    pub const fn selected_root(&self) -> &DurablePhysicalRootManifest {
        self.roster.selected_root()
    }
    pub const fn selected_root_sha256(&self) -> [u8; 32] {
        self.roster.selected_root_sha256()
    }
    pub const fn accumulator_v2(&self) -> ReleaseCheckpointAccumulatorV2 {
        self.roster.accumulator_v2()
    }
    pub fn batches(&self) -> &[ReleaseCheckpointBatchV1] {
        self.roster.batches()
    }
    pub fn selected_heads(&self) -> &[ReleaseCustodyHeadEntryV1] {
        self.roster.selected_heads()
    }
    pub const fn release_certificate_record_count(&self) -> u16 {
        self.roster.release_certificate_record_count()
    }
    pub const fn release_certificate_encoded_bytes(&self) -> u32 {
        self.roster.release_certificate_encoded_bytes()
    }
    pub const fn head_walk_frame_bytes(&self) -> u64 {
        self.roster.head_walk_frame_bytes()
    }
    pub const fn head_walk_peak_resident_bytes(&self) -> u64 {
        self.roster.head_walk_peak_resident_bytes()
    }

    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.roster.owned_heap_bytes()
    }

    pub const fn admission_peak_resident_bytes(&self) -> u64 {
        self.roster.admission_peak_resident_bytes()
    }
}
