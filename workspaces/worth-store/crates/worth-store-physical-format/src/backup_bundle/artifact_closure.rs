use sha2::{Digest, Sha256};

use super::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFamily, BackupBundleArtifactManifestRow,
};
use crate::{AllocationClassKind, PhysicalCellReuseDomain, PhysicalGenerationOwner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupBundlePhysicalOwner {
    domain: u8,
    segment: Option<u64>,
    page: Option<u64>,
    extent: Option<u64>,
    slot: Option<u64>,
    root: Option<u64>,
    allocation: u8,
    generation: u64,
}

impl BackupBundlePhysicalOwner {
    pub fn from_generation_owner(owner: PhysicalGenerationOwner) -> Self {
        Self {
            domain: domain_tag(owner.domain()),
            segment: owner.segment_id().map(|value| value.get()),
            page: owner.page_id().map(|value| value.get()),
            extent: owner.extent_id().map(|value| value.get()),
            slot: owner.slot().map(|value| u64::from(value.get())),
            root: owner.root_reference().map(|value| value.get()),
            allocation: owner.allocation_class().map_or(0, allocation_tag),
            generation: owner.generation().get(),
        }
    }

    pub(crate) fn is_valid(self) -> bool {
        self.generation > 0
            && match self.domain {
                1 => {
                    self.segment.is_some()
                        && self.page.is_some()
                        && self.slot.is_some()
                        && self.extent.is_none()
                        && self.root.is_none()
                        && self.allocation == 0
                }
                2 => {
                    self.segment.is_some()
                        && self.extent.is_some()
                        && self.page.is_none()
                        && self.slot.is_none()
                        && self.root.is_none()
                        && self.allocation == 0
                }
                7 => {
                    self.extent.is_some()
                        && self.segment.is_none()
                        && self.page.is_none()
                        && self.slot.is_none()
                        && self.root.is_none()
                        && self.allocation == 0
                }
                3 => {
                    self.segment.is_some()
                        && self.root.is_none()
                        && (1..=6).contains(&self.allocation)
                        && ((self.page.is_some() && self.slot.is_some() && self.extent.is_none())
                            || (self.extent.is_some()
                                && self.page.is_none()
                                && self.slot.is_none()))
                }
                4 => {
                    self.root.is_some()
                        && self.segment.is_none()
                        && self.page.is_none()
                        && self.extent.is_none()
                        && self.slot.is_none()
                        && self.allocation == 0
                }
                5 => {
                    self.segment.is_some()
                        && self.page.is_some()
                        && self.extent.is_none()
                        && self.slot.is_none()
                        && self.root.is_none()
                        && self.allocation == 0
                }
                6 => {
                    self.segment.is_some()
                        && self.page.is_none()
                        && self.extent.is_none()
                        && self.slot.is_none()
                        && self.root.is_none()
                        && self.allocation == 0
                }
                _ => false,
            }
    }

    pub fn generation_owner(self) -> Option<PhysicalGenerationOwner> {
        if !self.is_valid() {
            return None;
        }
        let generation = crate::PhysicalGeneration::from_raw(self.generation).ok()?;
        let authority = crate::PhysicalGenerationAuthority::for_canonical_physical_format();
        Some(match self.domain {
            1 => authority
                .slot_cell(
                    crate::PhysicalSegmentId::from_raw(self.segment?).ok()?,
                    crate::PhysicalPageId::from_raw(self.page?).ok()?,
                    crate::PhysicalRecordSlot::from_raw(u16::try_from(self.slot?).ok()?).ok()?,
                )
                .with_slot_generation(generation)
                .owner(),
            2 => authority
                .extent_cell(
                    crate::PhysicalSegmentId::from_raw(self.segment?).ok()?,
                    crate::PhysicalExtentId::from_raw(self.extent?).ok()?,
                )
                .with_extent_generation(generation)
                .owner(),
            7 => authority
                .record_extent_cell(crate::PhysicalExtentId::from_raw(self.extent?).ok()?)
                .with_extent_generation(generation)
                .owner(),
            4 => authority
                .root_publication_cell(crate::PhysicalRootReference::from_raw(self.root?).ok()?)
                .with_root_publication_generation(generation)
                .owner(),
            5 => authority
                .page_cell(
                    crate::PhysicalSegmentId::from_raw(self.segment?).ok()?,
                    crate::PhysicalPageId::from_raw(self.page?).ok()?,
                )
                .with_page_generation(generation)
                .owner(),
            6 => authority
                .segment_cell(crate::PhysicalSegmentId::from_raw(self.segment?).ok()?)
                .with_segment_generation(generation)
                .owner(),
            _ => return None,
        })
    }

    pub(crate) fn matches_artifact(
        self,
        family: BackupBundleArtifactFamily,
        generation: u64,
    ) -> bool {
        self.generation == generation
            && matches!(
                (family, self.domain),
                (
                    BackupBundleArtifactFamily::RootManifest
                        | BackupBundleArtifactFamily::SecondaryRoot,
                    4
                ) | (BackupBundleArtifactFamily::WalSegment, 6)
                    | (
                        BackupBundleArtifactFamily::Extent | BackupBundleArtifactFamily::BlobChunk,
                        2
                    )
                    | (BackupBundleArtifactFamily::Extent, 7)
                    | (
                        BackupBundleArtifactFamily::CheckpointManifest
                            | BackupBundleArtifactFamily::Index,
                        1
                    )
                    | (BackupBundleArtifactFamily::Page, 5)
            )
    }

    pub(crate) fn update_digest(self, digest: &mut Sha256) {
        digest.update([self.domain]);
        for value in [self.segment, self.page, self.extent, self.slot, self.root] {
            digest.update(value.unwrap_or(0).to_le_bytes());
        }
        digest.update(self.generation.to_le_bytes());
        digest.update([self.allocation]);
    }
}

pub fn backup_canonical_artifact_closure_digest<'a>(
    rows: impl IntoIterator<Item = &'a BackupBundleArtifactManifestRow>,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    for row in rows {
        digest.update([row.family() as u8]);
        digest.update([row.format() as u8]);
        digest.update(row.identity().as_bytes());
        digest.update(row.generation().to_le_bytes());
        digest.update(row.bytes().to_le_bytes());
        digest.update(row.content_digest());
        row.reclaim_owner().update_digest(&mut digest);
        update_coverage_digest(&mut digest, row.coverage());
    }
    digest.finalize().into()
}

fn update_coverage_digest(digest: &mut Sha256, coverage: &BackupBundleArtifactCoverage) {
    match coverage {
        BackupBundleArtifactCoverage::RootManifest { root_generation } => {
            digest.update([1]);
            digest.update(root_generation.to_le_bytes());
        }
        BackupBundleArtifactCoverage::CheckpointManifest {
            checkpoint_identity,
            manifest_generation,
            durable_checkpoint_lsn,
            authority_fingerprint,
            frontier_digest,
        } => {
            digest.update([2]);
            digest.update(checkpoint_identity.as_bytes());
            digest.update(manifest_generation.to_le_bytes());
            digest.update(durable_checkpoint_lsn.to_le_bytes());
            digest.update(authority_fingerprint);
            digest.update(frontier_digest);
        }
        BackupBundleArtifactCoverage::WalSegment {
            start_lsn,
            end_exclusive_lsn,
        } => {
            digest.update([3]);
            digest.update(start_lsn.to_le_bytes());
            digest.update(end_exclusive_lsn.to_le_bytes());
        }
        BackupBundleArtifactCoverage::PhysicalReachability => digest.update([4]),
        BackupBundleArtifactCoverage::SecondaryRoot { root_generation } => {
            digest.update([5]);
            digest.update(root_generation.to_le_bytes());
        }
    }
}

const fn domain_tag(domain: PhysicalCellReuseDomain) -> u8 {
    match domain {
        PhysicalCellReuseDomain::SlotAllocation => 1,
        PhysicalCellReuseDomain::ExtentAllocation => 2,
        PhysicalCellReuseDomain::RecordExtentAllocation => 7,
        PhysicalCellReuseDomain::FreeSpaceReuse => 3,
        PhysicalCellReuseDomain::RootPublication => 4,
        PhysicalCellReuseDomain::Page => 5,
        PhysicalCellReuseDomain::Segment => 6,
    }
}

const fn allocation_tag(class: AllocationClassKind) -> u8 {
    match class {
        AllocationClassKind::OrdinaryRecordPage => 1,
        AllocationClassKind::LargeRecordExtent => 2,
        AllocationClassKind::RootManifest => 3,
        AllocationClassKind::SegmentManifest => 4,
        AllocationClassKind::ExtentManifest => 5,
        AllocationClassKind::FreeSpaceMap => 6,
    }
}
