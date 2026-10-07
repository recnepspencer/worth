//! Certification orchestration through the ordinary consuming COW insertion.

use sha2::{Digest, Sha256};
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    BlobGenerationPublicationV1, IndexedThroughBlobPublication, PersistedRecordIdentity,
};

use crate::physical_runtime::{
    layout::{PhysicalIndexPointKey, PhysicalLayoutPagePort},
    AdmittedRecordPlacementPolicy, MaintenancePhysicalAllocation, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};

use super::super::{
    insert_registered_node, tree::InsertionSource, PhysicalLayoutMaintenanceFailure,
};

impl ServingPhysicalRuntime {
    /// Builds an unpublished COW catalog from three genuine selected
    /// publications. The callback observes the two completed appends and may
    /// retain an actual pool allocation across the third insertion. This does
    /// not publish a directory or manufacture a serving/recovery authority.
    pub fn certification_try_selected_catalog_continuation<'runtime>(
        &'runtime self,
        sources: [IndexedThroughBlobPublication; 3],
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        before_continuation: impl FnOnce() -> Option<MaintenancePhysicalAllocation<'runtime>>,
    ) -> Result<PersistedRecordIdentity, PhysicalLayoutMaintenanceFailure> {
        if sources[0].record() == sources[1].record()
            || sources[0].record() == sources[2].record()
            || sources[1].record() == sources[2].record()
        {
            return Err(PhysicalLayoutMaintenanceFailure::SourceIdentityMismatch);
        }
        let mut callback = Some(before_continuation);
        let mut collision = None;
        let mut chain = None;
        for (index, source) in sources.into_iter().enumerate() {
            // Refresh the selected reader so it routes the preceding actual
            // classified C.5 append. Construct the port before pressure.
            let reader = self
                .records()
                .map_err(PhysicalLayoutMaintenanceFailure::RootProtection)?;
            let port = PhysicalLayoutPagePort::from_protected_reader(
                self,
                reader,
                self.maximum_inline_record_bytes(),
            )
            .map_err(PhysicalLayoutMaintenanceFailure::SourceRead)?;
            let read = port
                .read_node(source.record())
                .map_err(PhysicalLayoutMaintenanceFailure::SourceRead)?;
            let digest: [u8; 32] = Sha256::digest(read.bytes()).into();
            if digest != source.encoded_digest() {
                return Err(PhysicalLayoutMaintenanceFailure::SourceDigestMismatch);
            }
            let publication = BlobGenerationPublicationV1::decode(read.bytes())
                .map_err(PhysicalLayoutMaintenanceFailure::SourceFormat)?;
            if publication.store() != self.store_identity().bytes() {
                return Err(PhysicalLayoutMaintenanceFailure::SourceIdentityMismatch);
            }
            let key = PhysicalIndexPointKey::selected_blob_catalog(
                self.store_identity(),
                publication.object(),
                publication.generation(),
            )
            .map_err(PhysicalLayoutMaintenanceFailure::PointKey)?;
            let key = key.canonical_bytes().to_vec();
            let value = super::super::encode_record(source.record());
            drop(read);
            if index == 2 {
                collision = callback.take().expect("single continuation observation")();
            }
            let source = match chain.take() {
                Some(prior) => InsertionSource::Continue(prior),
                None => InsertionSource::Root(None),
            };
            chain = Some(insert_registered_node(
                self,
                &port,
                DurableArtifactFamilyId::BlobCatalog,
                source,
                key,
                value,
                placement,
                deadline,
            )?);
        }
        // The observation grant stays live through admission and effects,
        // including the failure return's ordinary lexical disposal.
        drop(collision);
        Ok(chain.expect("three real catalog insertions").root())
    }
}
