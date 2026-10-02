//! Exact C.9-observed routing and extent frames retained across media-owner
//! handoff. The Serving owner rereads these same selected ranges before open.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{ArtifactTreeDirectory, QualifiedFilesystemMedia};
use worth_store_physical_format::RecordArtifactFile;

use super::super::resident::{PhysicalRecoveryRejoinResidentDenial, StoreRejoinResidentLedger};
use super::super::{release_heads::FundedHeadSlices, SelectedMediaRejoinDenial};

mod funded_heads;
use funded_heads::SelectedHeadMediaWitness;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) struct SelectedArtifactSlice {
    artifact: RecordArtifactFile,
    offset: u64,
    length: u32,
    sha256: [u8; 32],
    whole_file: bool,
}

impl SelectedArtifactSlice {
    pub(in crate::physical_runtime) fn observed(
        artifact: RecordArtifactFile,
        offset: u64,
        bytes: &[u8],
        whole_file: bool,
    ) -> Option<Self> {
        let length = u32::try_from(bytes.len()).ok()?;
        (length != 0 && offset.checked_add(u64::from(length)).is_some()).then(|| Self {
            artifact,
            offset,
            length,
            sha256: Sha256::digest(bytes).into(),
            whole_file,
        })
    }

    fn matches_serving_media(&self, media: &QualifiedFilesystemMedia) -> bool {
        let Some(directory) = artifact_directory(self.artifact) else {
            return false;
        };
        let Ok(file) = directory.file(&self.artifact.file_name()) else {
            return false;
        };
        let tree = media.artifact_tree();
        if self.whole_file && tree.file_length(&file).ok() != Some(u64::from(self.length)) {
            return false;
        }
        let mut bytes = vec![0; self.length as usize];
        if tree.read_exact_at(&file, self.offset, &mut bytes).is_err() {
            return false;
        }
        <[u8; 32]>::from(Sha256::digest(bytes)) == self.sha256
    }
}

pub(in crate::physical_runtime) struct SelectedControlMediaFingerprint {
    slices: Vec<SelectedArtifactSlice>,
    heads: SelectedHeadMediaWitness,
}

impl SelectedControlMediaFingerprint {
    pub(in crate::physical_runtime) fn observed(slices: Vec<SelectedArtifactSlice>) -> Self {
        Self {
            slices,
            heads: SelectedHeadMediaWitness::Absent,
        }
    }

    pub(in crate::physical_runtime) fn selected_heads(heads: FundedHeadSlices) -> Self {
        Self {
            slices: Vec::new(),
            heads: SelectedHeadMediaWitness::Selected(heads),
        }
    }

    pub(in crate::physical_runtime) fn pending_heads(
        checkpoint: FundedHeadSlices,
        pre_pending: FundedHeadSlices,
        effective: FundedHeadSlices,
    ) -> Self {
        Self {
            slices: Vec::new(),
            heads: SelectedHeadMediaWitness::PendingReplay {
                checkpoint,
                pre_pending,
                effective,
            },
        }
    }

    /// Actual retained backing, distinct from the conservative rejoin bound.
    pub(in crate::physical_runtime) fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.slices.capacity())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<SelectedArtifactSlice>()).ok()?)?
            .checked_add(self.heads.owned_heap_bytes()?)
    }

    pub(in crate::physical_runtime) fn independently_funded_heap_bytes(&self) -> Option<u64> {
        self.heads.owned_heap_bytes()
    }

    pub(in crate::physical_runtime) fn has_selected_head_walk(&self) -> bool {
        matches!(self.heads, SelectedHeadMediaWitness::Selected(_))
    }

    pub(in crate::physical_runtime) fn has_pending_head_walks(&self) -> bool {
        matches!(self.heads, SelectedHeadMediaWitness::PendingReplay { .. })
    }

    pub(in crate::physical_runtime) fn has_no_head_walk(&self) -> bool {
        matches!(self.heads, SelectedHeadMediaWitness::Absent)
    }

    pub(in crate::physical_runtime) fn verify_funded_heads_for_serving(
        &self,
        media: &QualifiedFilesystemMedia,
        window: &mut crate::physical_runtime::PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<bool, crate::physical_runtime::record_serving::RecordBootstrapDenial> {
        self.heads.verify_serving_media(media, window)
    }

    pub(in crate::physical_runtime) fn matches_serving_media(
        &self,
        media: &QualifiedFilesystemMedia,
    ) -> bool {
        self.slices
            .iter()
            .all(|slice| slice.matches_serving_media(media))
    }

    pub(in crate::physical_runtime) fn extend(
        &mut self,
        other: Self,
    ) -> Result<(), SelectedMediaRejoinDenial> {
        if !self.heads.can_merge(&other.heads) {
            return Err(SelectedMediaRejoinDenial::CertificateRoster);
        }
        self.slices.extend(other.slices);
        self.heads.merge_admitted(other.heads);
        Ok(())
    }

    /// Both fingerprints' backing is already retained in this same ledger.
    /// Growth keeps the donor and previous destination charged through the
    /// reallocation, then releases the donor only after its backing is dropped.
    pub(in crate::physical_runtime) fn extend_with_resident(
        &mut self,
        mut other: Self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(), SelectedMediaRejoinDenial> {
        if !self.heads.can_merge(&other.heads) {
            return Err(SelectedMediaRejoinDenial::CertificateRoster);
        }
        let donor_bytes = resident
            .vector_bytes(&other.slices)
            .map_err(SelectedMediaRejoinDenial::Resident)?;
        resident
            .grow_vec(&mut self.slices, other.slices.len())
            .map_err(SelectedMediaRejoinDenial::Resident)?;
        self.slices.append(&mut other.slices);
        self.heads.merge_admitted(std::mem::replace(
            &mut other.heads,
            SelectedHeadMediaWitness::Absent,
        ));
        drop(other);
        resident.release(donor_bytes);
        Ok(())
    }

    pub(in crate::physical_runtime) fn retained_memory_bytes(&self) -> u64 {
        (self.slices.capacity() as u64)
            .saturating_mul(4 * std::mem::size_of::<SelectedArtifactSlice>() as u64)
            .saturating_add(
                self.heads
                    .owned_heap_bytes()
                    .unwrap_or(u64::MAX)
                    .saturating_mul(4),
            )
    }

    pub(in crate::physical_runtime) fn try_extend_bounded(
        &mut self,
        other: Self,
        maximum_retained_bytes: u64,
    ) -> bool {
        if !self.heads.can_merge(&other.heads) {
            return false;
        }
        let head_bytes = match self.heads.owned_heap_bytes().and_then(|bytes| {
            bytes
                .checked_add(other.heads.owned_heap_bytes()?)?
                .checked_mul(4)
        }) {
            Some(bytes) => bytes,
            None => return false,
        };
        let next_len = match self.slices.len().checked_add(other.slices.len()) {
            Some(next) => next,
            None => return false,
        };
        let needed = match (next_len as u64)
            .checked_mul(4 * std::mem::size_of::<SelectedArtifactSlice>() as u64)
            .and_then(|bytes| bytes.checked_add(head_bytes))
        {
            Some(bytes) if bytes <= maximum_retained_bytes => bytes,
            _ => return false,
        };
        if self.slices.try_reserve_exact(other.slices.len()).is_err()
            || self.retained_memory_bytes() > maximum_retained_bytes
            || needed > maximum_retained_bytes
        {
            return false;
        }
        self.slices.extend(other.slices);
        self.heads.merge_admitted(other.heads);
        self.retained_memory_bytes() <= maximum_retained_bytes
    }
}

#[cfg(test)]
#[path = "snapshot/resident_merge_tests.rs"]
mod resident_merge_tests;

fn artifact_directory(artifact: RecordArtifactFile) -> Option<ArtifactTreeDirectory> {
    let records = ArtifactTreeDirectory::families().child("records").ok()?;
    match artifact {
        RecordArtifactFile::CurrentRootSelector => Some(records),
        RecordArtifactFile::RootManifest { .. }
        | RecordArtifactFile::RootRoutingBlock { .. }
        | RecordArtifactFile::ReleaseCustodyHeadBlock { .. } => records.child("roots").ok(),
        RecordArtifactFile::SegmentMembershipBlock { .. } => {
            records.child("segment-manifests").ok()
        }
        RecordArtifactFile::FreeSpaceManifest { .. }
        | RecordArtifactFile::FreeSpaceMembershipBlock { .. } => records.child("free-space").ok(),
        RecordArtifactFile::ExtentArena { .. } => records.child("arenas").ok(),
        _ => None,
    }
}
