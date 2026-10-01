//! Authenticate the published generation named by each selected V3 manifest.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, BlobReclaimSourceKind,
    PersistedRecordIdentity,
};

use super::{
    BoundedRecoveryFilesystemDiscovery, ManifestEntryBudget, PhysicalRecordFormatDeclaration,
    RecoveryIntegrityIngressTrace, SelectedReleaseJoin,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelectedReleasedGeneration {
    pub(crate) object: [u8; 16],
    pub(crate) generation: u64,
    pub(crate) publication: PersistedRecordIdentity,
    pub(crate) publication_digest: [u8; 32],
}

impl SelectedReleaseJoin<'_> {
    pub(crate) fn source_generation(
        &self,
        descriptor: BlobReclaimDescriptorV3,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        budget: &mut ManifestEntryBudget,
        trace: &mut RecoveryIntegrityIngressTrace,
        scratch: &mut u64,
    ) -> Option<SelectedReleasedGeneration> {
        let base = descriptor.base();
        let (manifest, frame) = self.read_manifest(
            discovery,
            format,
            budget,
            trace,
            scratch,
            base.manifest_record(),
        )?;
        if <[u8; 32]>::from(Sha256::digest(&frame)) != base.manifest_frame_sha256()
            || manifest.store() != self.store
            || manifest.reclaim_attempt() != base.reclaim_attempt()
            || manifest.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
            || manifest.source_basis_digest() != base.source_basis_digest()
            || manifest.count() != base.manifest_count()
        {
            return None;
        }
        let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
            return None;
        };
        Some(SelectedReleasedGeneration {
            object: source.object(),
            generation: source.generation(),
            publication: source.publication_record(),
            publication_digest: source.publication_frame_sha256(),
        })
    }
}
