use worth_store_physical_backend::CompletedScheduledRecoveryReopenRead;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurablePhysicalRootManifest,
    PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_physical_integrity::{
    validate_root_manifest, IntegrityValidatedRootManifest, PhysicalArtifactScope,
    PhysicalByteRange, RootManifestIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::completed_read::{BoundScheduledRootProtocolSource, ScheduledRootProtocolSource};
use crate::physical_runtime::RootProtocolAdmissionDenial;

pub(in crate::physical_runtime::recovery_coordination) struct IntegrityAdmittedRootManifest<'source>
{
    source: ScheduledRootProtocolSource<'source>,
    projection: AdmittedRootManifestProjection,
}

#[derive(Clone, Copy)]
struct AdmittedRootManifestProjection {
    generation: u64,
    tree_identity: u64,
    node_capacity: u16,
    free_space_checksum: u32,
    record_count: u64,
    next_block: u64,
    next_segment_block: u64,
    routing_root: Option<worth_store_physical_format::ManifestBlockReference>,
    segment_root: Option<worth_store_physical_format::SegmentManifestBlockReference>,
    free_space_root: Option<worth_store_physical_format::FreeSpaceBlockReference>,
    release_custody_head_root:
        Option<worth_store_physical_format::ReleaseCustodyHeadBlockReferenceV1>,
    next_release_custody_head_block: u64,
    latest_blob_publication: Option<worth_store_physical_format::IndexedThroughBlobPublication>,
    latest_blob_quarantine: Option<worth_store_physical_format::PersistedRecordIdentity>,
    tier_epoch_anchor: Option<[u8; 32]>,
    derived_family_directory:
        Option<worth_store_physical_format::DerivedFamilyRootDirectoryBinding>,
    last_inline_record: Option<worth_store_physical_format::PersistedRecordIdentity>,
    last_inline_segment: Option<worth_store_physical_format::SegmentGenerationCell>,
    requires_maintenance_protocol: bool,
}

pub(in crate::physical_runtime::recovery_coordination) fn admit_scheduled_root_manifest(
    read: &CompletedScheduledRecoveryReopenRead,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
) -> Result<IntegrityAdmittedRootManifest<'_>, RootProtocolAdmissionDenial> {
    let expected = RecordArtifactFile::RootManifest { generation };
    if read.artifact() != expected {
        return Err(RootProtocolAdmissionDenial::SourceArtifactMismatch);
    }
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(read.bytes());
    let scope = manifest_scope(store, format, generation, read.bytes().len() as u64)?;
    let source = BoundScheduledRootProtocolSource::bind(read, expected, scope)?;
    let validated = validate(input, scope)?;
    if !validated.matches_input(input) {
        return Err(RootProtocolAdmissionDenial::SourceIncarnationMismatch);
    }
    let projection = projection(&validated);
    let source = source.admit(validated.into_validation_record())?;
    Ok(IntegrityAdmittedRootManifest { source, projection })
}

fn validate<'source>(
    input: UntrustedPhysicalArtifact<'source>,
    scope: PhysicalArtifactScope,
) -> Result<IntegrityValidatedRootManifest<'source>, RootProtocolAdmissionDenial> {
    match validate_root_manifest(input, scope).0 {
        RootManifestIntegrityValidation::Intact(validated) => Ok(validated),
        RootManifestIntegrityValidation::Rejected(rejection) => {
            Err(RootProtocolAdmissionDenial::from_validation(rejection))
        }
    }
}

fn manifest_scope(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
    length: u64,
) -> Result<PhysicalArtifactScope, RootProtocolAdmissionDenial> {
    let range = PhysicalByteRange::new(0, length)
        .map_err(|_| RootProtocolAdmissionDenial::SourceRangeMismatch)?;
    PhysicalArtifactScope::root_manifest(store, format, generation, range)
        .map_err(|_| RootProtocolAdmissionDenial::SourceArtifactMismatch)
}

fn projection(validated: &IntegrityValidatedRootManifest<'_>) -> AdmittedRootManifestProjection {
    AdmittedRootManifestProjection {
        generation: validated.root_generation(),
        tree_identity: validated.tree_identity(),
        node_capacity: validated.node_capacity(),
        free_space_checksum: validated.free_space_checksum(),
        record_count: validated.record_count(),
        next_block: validated.next_block(),
        next_segment_block: validated.next_segment_block(),
        routing_root: validated.routing_root(),
        segment_root: validated.segment_root(),
        free_space_root: validated.free_space_root(),
        release_custody_head_root: validated.release_custody_head_root(),
        next_release_custody_head_block: validated.next_release_custody_head_block(),
        latest_blob_publication: validated.latest_blob_publication(),
        latest_blob_quarantine: validated.latest_blob_quarantine(),
        tier_epoch_anchor: validated.tier_epoch_anchor(),
        derived_family_directory: validated.derived_family_directory(),
        last_inline_record: validated.last_inline_record(),
        last_inline_segment: validated.last_inline_segment(),
        requires_maintenance_protocol: validated.requires_maintenance_protocol(),
    }
}

impl IntegrityAdmittedRootManifest<'_> {
    pub(in crate::physical_runtime::recovery_coordination) fn project(
        self,
    ) -> Result<DurablePhysicalRootManifest, RootProtocolAdmissionDenial> {
        let _source_incarnation = (self.source.operation(), self.source.validation());
        self.projection.project()
    }
}

impl AdmittedRootManifestProjection {
    fn project(self) -> Result<DurablePhysicalRootManifest, RootProtocolAdmissionDenial> {
        DurablePhysicalRootManifest::builder(
            self.generation,
            self.tree_identity,
            self.node_capacity,
            self.free_space_checksum,
        )
        .record_count(self.record_count)
        .next_block(self.next_block)
        .next_segment_block(self.next_segment_block)
        .routing_root(self.routing_root)
        .segment_root(self.segment_root)
        .free_space_root(self.free_space_root)
        .release_custody_head_root(self.release_custody_head_root)
        .next_release_custody_head_block(self.next_release_custody_head_block)
        .latest_blob_publication(self.latest_blob_publication)
        .latest_blob_quarantine(self.latest_blob_quarantine)
        .tier_epoch_anchor(self.tier_epoch_anchor)
        .derived_family_directory(self.derived_family_directory)
        .last_inline_record(self.last_inline_record)
        .last_inline_segment(self.last_inline_segment)
        .admit()
        .map(|root| {
            if self.requires_maintenance_protocol {
                root.with_maintenance_protocol()
            } else {
                root
            }
        })
        .ok_or(RootProtocolAdmissionDenial::OwnerProjectionRejected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{
        store_namespace::{
            ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
        },
        ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadKeyV1,
    };

    #[test]
    fn scheduled_owner_projection_preserves_schema_ten_head_and_maintenance() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([31; 16]).unwrap(),
        )
        .published_identity();
        let key = ReleaseCustodyHeadKeyV1::new([17; 16], 2).unwrap();
        let head = ReleaseCustodyHeadBlockReferenceV1::new(7, 3, 0, key, key, [29; 32]).unwrap();
        let expected = DurablePhysicalRootManifest::builder(7, 11, 2, 43)
            .release_custody_head_root(Some(head))
            .next_release_custody_head_block(4)
            .admit()
            .unwrap()
            .with_maintenance_protocol();
        let bytes = expected.encode(format);
        assert_eq!(bytes[9], 10);
        let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
        let scope = manifest_scope(store, format, 7, bytes.len() as u64).unwrap();
        let validated = validate(input, scope).unwrap();
        assert!(validated.matches_input(input));
        assert_eq!(validated.release_custody_head_root(), Some(head));
        assert_eq!(validated.next_release_custody_head_block(), 4);
        let observed = projection(&validated).project().unwrap();
        assert_eq!(observed, expected);
        assert!(observed.requires_maintenance_protocol());
    }
}
