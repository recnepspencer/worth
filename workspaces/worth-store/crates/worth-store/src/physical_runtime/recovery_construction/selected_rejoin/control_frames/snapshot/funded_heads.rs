//! Named full-tree observations retain native custody across recovery handoff.
//! Effect witnesses and routing/control slices have separate ownership.

use worth_store_physical_backend::QualifiedFilesystemMedia;

use super::{FundedHeadSlices, SelectedArtifactSlice};
use crate::physical_runtime::{
    record_serving::RecordBootstrapDenial, PhysicalRecoveryReadAllocation,
};

pub(super) enum SelectedHeadMediaWitness {
    Absent,
    Selected(FundedHeadSlices),
    PendingReplay {
        checkpoint: FundedHeadSlices,
        pre_pending: FundedHeadSlices,
        effective: FundedHeadSlices,
    },
}

impl SelectedHeadMediaWitness {
    fn components(&self) -> impl Iterator<Item = &FundedHeadSlices> {
        let components = match self {
            Self::Absent => [None, None, None],
            Self::Selected(heads) => [Some(heads), None, None],
            Self::PendingReplay {
                checkpoint,
                pre_pending,
                effective,
            } => [Some(checkpoint), Some(pre_pending), Some(effective)],
        };
        components.into_iter().flatten()
    }

    pub(super) fn owned_heap_bytes(&self) -> Option<u64> {
        self.components().try_fold(0_u64, |bytes, heads| {
            bytes.checked_add(heads.owned_heap_bytes()?)
        })
    }

    pub(super) fn can_merge(&self, other: &Self) -> bool {
        matches!(self, Self::Absent) || matches!(other, Self::Absent)
    }

    pub(super) fn merge_admitted(&mut self, other: Self) {
        match (&*self, other) {
            (_, Self::Absent) => {}
            (Self::Absent, heads) => *self = heads,
            _ => unreachable!("head mode compatibility is admitted before storage mutation"),
        }
    }

    pub(super) fn verify_serving_media(
        &self,
        media: &QualifiedFilesystemMedia,
        window: &mut PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<bool, RecordBootstrapDenial> {
        let mut entries = 0_u64;
        let mut bytes = 0_u64;
        for heads in self.components() {
            if !heads.matching_owner(window) {
                return Err(RecordBootstrapDenial::RecoveredHeadWitnessOwnerMismatch);
            }
            for slice in heads.slices() {
                entries = entries
                    .checked_add(1)
                    .ok_or(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)?;
                bytes = bytes
                    .checked_add(u64::from(slice.length))
                    .ok_or(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)?;
            }
        }
        if entries == 0 {
            return Ok(true);
        }
        let mut observation = media
            .bounded_record_observation(entries, bytes)
            .map_err(RecordBootstrapDenial::RecoveredHeadObservationUnavailable)?;
        for heads in self.components() {
            for slice in heads.slices() {
                if !slice.matches_funded_serving_media(&mut observation, window)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

impl SelectedArtifactSlice {
    fn matches_funded_serving_media(
        &self,
        media: &mut worth_store_physical_backend::BorrowedRecordFilesystemObservation<'_>,
        window: &mut PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<bool, RecordBootstrapDenial> {
        use sha2::{Digest, Sha256};
        let observation = if self.whole_file {
            window.read_serving_record(media, self.artifact, u64::from(self.length))
        } else {
            window.read_serving_record_range(
                media,
                self.artifact,
                self.offset,
                self.length,
                u64::from(self.length),
            )
        }
        .map_err(RecordBootstrapDenial::RecoveredHeadRead)?;
        let Some(bytes) = observation.observed().bytes() else {
            return Ok(false);
        };
        Ok(bytes.len() == self.length as usize
            && <[u8; 32]>::from(Sha256::digest(bytes)) == self.sha256)
    }
}
