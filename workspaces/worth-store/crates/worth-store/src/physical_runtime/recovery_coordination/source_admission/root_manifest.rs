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
    last_inline_record: Option<worth_store_physical_format::PersistedRecordIdentity>,
    last_inline_segment: Option<worth_store_physical_format::SegmentGenerationCell>,
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
        last_inline_record: validated.last_inline_record(),
        last_inline_segment: validated.last_inline_segment(),
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
        .last_inline_record(self.last_inline_record)
        .last_inline_segment(self.last_inline_segment)
        .admit()
        .ok_or(RootProtocolAdmissionDenial::OwnerProjectionRejected)
    }
}
