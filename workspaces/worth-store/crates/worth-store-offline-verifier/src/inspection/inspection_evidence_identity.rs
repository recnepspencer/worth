use crate::OfflinePhysicalArtifactFamily;
use sha2::{Digest, Sha256};

use super::{OfflineInspectionCounters, OfflineStructuralIdentification, StructurallyWalkedMedia};

impl StructurallyWalkedMedia {
    /// Stable evidence identity for the completed bounded, read-only media walk.
    ///
    /// The identity deliberately excludes absolute paths and OS file ids so a
    /// clean-checkout fixture run remains comparable while still binding the
    /// consistency basis, every observed byte range, owner classification, and
    /// exact inspection counters.
    pub fn inspection_evidence_identity(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth-store-offline-inspection-evidence-v1");
        update_text(&mut digest, self.consistency_basis().identity());
        digest.update((self.files().len() as u64).to_be_bytes());
        for file in self.files() {
            digest.update((file.source_index() as u64).to_be_bytes());
            digest.update(file.length().to_be_bytes());
            digest.update([family_tag(file.family())]);
            digest.update([identification_tag(file.structural_identification())]);
            update_optional_u64(&mut digest, file.generation());
            match file.physical_owner() {
                Some(owner) => {
                    digest.update([1]);
                    digest.update(owner.stable_fingerprint());
                }
                None => digest.update([0]),
            }
            digest.update(file.content_digest());
            digest.update(file.source().metadata_fingerprint());
            digest.update(file.source().physical_alias_group().to_be_bytes());
        }
        update_counters(&mut digest, self.counters());
        digest.finalize().into()
    }
}

fn update_counters(digest: &mut Sha256, counters: OfflineInspectionCounters) {
    for value in [
        counters.backend_requested_bytes(),
        counters.bytes_read(),
        counters.peak_buffer_bytes(),
        counters.peak_owned_allocation_bytes(),
        counters.decoder_allocated_bytes(),
        counters.file_touches(),
        counters.chunk_touches(),
        counters.checkpoint_revalidated_files(),
        counters.checkpoint_revalidated_bytes(),
        counters.checkpoint_rejections(),
    ] {
        digest.update(value.to_be_bytes());
    }
}

fn update_text(digest: &mut Sha256, value: &str) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value.as_bytes());
}

fn update_optional_u64(digest: &mut Sha256, value: Option<u64>) {
    match value {
        Some(value) => {
            digest.update([1]);
            digest.update(value.to_be_bytes());
        }
        None => digest.update([0]),
    }
}

const fn family_tag(family: OfflinePhysicalArtifactFamily) -> u8 {
    match family {
        OfflinePhysicalArtifactFamily::Manifest => 1,
        OfflinePhysicalArtifactFamily::Page => 2,
        OfflinePhysicalArtifactFamily::Extent => 3,
        OfflinePhysicalArtifactFamily::Wal => 4,
        OfflinePhysicalArtifactFamily::Index => 5,
        OfflinePhysicalArtifactFamily::BlobChunk => 6,
        OfflinePhysicalArtifactFamily::Unknown => 7,
    }
}

const fn identification_tag(identification: OfflineStructuralIdentification) -> u8 {
    match identification {
        OfflineStructuralIdentification::FileNameHint => 1,
        OfflineStructuralIdentification::OwnerDecoded => 2,
    }
}
