use std::path::Path;

use worth_store_contracts::{
    AcceptedHandoffReadiness, HandoffEvidenceDigestSet, StableDigest, ROADMAP_2_S1_SCOPE,
};
use worth_store_physical_format::{
    InMemoryPhysicalFormatModel, InMemoryPhysicalFormatModelRequest, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalStoreIdentity, PlatformPhysicalAppendRequest,
};
#[cfg(test)]
use worth_store_physical_format::{
    PhysicalBinaryEncodingWitness, PhysicalHeaderAuthority, PhysicalPageKind,
    PHYSICAL_HEADER_LENGTH,
};
use worth_store_physical_isolation::BackupArtifactReference;
#[cfg(test)]
use worth_store_physical_isolation::CurrentGenerationPhysicalReference;

mod canonical_materialization;
mod physical_reference;
use canonical_materialization::{
    materialize_canonical_backup_artifacts, CanonicalBackupArtifactRequest,
};
use physical_reference::*;

pub(crate) struct CanonicalBackupArtifacts {
    pub(crate) references: Vec<BackupArtifactReference>,
    pub(crate) checkpoint_identity: String,
}

#[cfg(test)]
pub(crate) fn canonical_backup_artifacts_at_root_generation(
    case: &str,
    source: &Path,
    root_generation: u64,
    store_identity: PhysicalStoreIdentity,
) -> CanonicalBackupArtifacts {
    canonical_backup_artifacts_with_blob_count(case, source, root_generation, 1, store_identity)
}

pub(crate) fn canonical_backup_artifacts_with_blob_count(
    case: &str,
    source: &Path,
    root_generation: u64,
    blob_count: u64,
    store_identity: PhysicalStoreIdentity,
) -> CanonicalBackupArtifacts {
    assert!(root_generation > 0, "fixture root generation is nonzero");
    assert!(blob_count > 0, "fixture blob count is nonzero");
    let mut world = CanonicalBackupArtifactWorld::new(case, blob_count, store_identity);
    let mut artifacts = None;
    for _ in 0..root_generation {
        artifacts = Some(world.publish(source));
    }
    artifacts.expect("nonzero publication count")
}

#[cfg(test)]
pub(crate) fn canonical_backup_artifacts_across_one_root_publication(
    case: &str,
    older_source: &Path,
    newer_source: &Path,
    store_identity: PhysicalStoreIdentity,
) -> (CanonicalBackupArtifacts, CanonicalBackupArtifacts) {
    let mut world = CanonicalBackupArtifactWorld::new(case, 1, store_identity);
    let older = world.publish(older_source);
    let newer = world.publish(newer_source);
    (older, newer)
}

struct CanonicalBackupArtifactWorld {
    case: String,
    runtime: InMemoryPhysicalFormatModel,
    generation: PhysicalGeneration,
    blob_count: u64,
}

impl CanonicalBackupArtifactWorld {
    fn new(case: &str, blob_count: u64, store_identity: PhysicalStoreIdentity) -> Self {
        let mut runtime = open_physical_runtime_for_store(store_identity);
        let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
        let generation = PhysicalGeneration::from_raw(1).expect("generation");
        let slot = generations
            .slot_cell(segment(200), page(4), record_slot(1))
            .with_slot_generation(generation);
        runtime
            .append_physical_record(PlatformPhysicalAppendRequest::page_slot(
                slot,
                format!("{case}:page-record").as_bytes(),
            ))
            .expect("physical page append");
        let appended_extent = generations
            .extent_cell(segment(100), extent(5))
            .with_extent_generation(generation);
        runtime
            .append_physical_record(PlatformPhysicalAppendRequest::extent(
                appended_extent,
                format!("{case}:extent-record").as_bytes(),
            ))
            .expect("physical extent append");
        Self {
            case: case.to_owned(),
            runtime,
            generation,
            blob_count,
        }
    }

    fn publish(&mut self, source: &Path) -> CanonicalBackupArtifacts {
        let publication = self
            .runtime
            .publish_physical_root()
            .expect("root publication");
        materialize_canonical_backup_artifacts(CanonicalBackupArtifactRequest {
            case: &self.case,
            source,
            runtime: &self.runtime,
            publication,
            generation: self.generation,
            blob_count: self.blob_count,
        })
    }
}

#[cfg(test)]
pub(crate) fn open_physical_runtime() -> InMemoryPhysicalFormatModel {
    InMemoryPhysicalFormatModel::start_empty_model(
        physical_readiness(),
        InMemoryPhysicalFormatModelRequest::physical_format_canonical(),
    )
    .expect("physical runtime")
}

fn open_physical_runtime_for_store(
    store_identity: PhysicalStoreIdentity,
) -> InMemoryPhysicalFormatModel {
    InMemoryPhysicalFormatModel::start_empty_model(
        physical_readiness(),
        InMemoryPhysicalFormatModelRequest::physical_format_for_store(store_identity),
    )
    .expect("physical runtime for store")
}

#[cfg(test)]
pub(crate) fn copy_page_to_owner(
    source: &Path,
    target: &Path,
    owner: CurrentGenerationPhysicalReference,
) {
    let owner = owner.owner();
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(
            owner.segment_id().expect("page segment"),
            owner.page_id().expect("page id"),
        )
        .with_page_generation(owner.generation());
    let source_bytes = std::fs::read(source).expect("source page bytes");
    let payload = source_bytes
        .get(usize::from(PHYSICAL_HEADER_LENGTH)..)
        .expect("canonical page header");
    let binary = PhysicalBinaryEncodingWitness::physical_format_canonical().expect("encoding");
    let headers = PhysicalHeaderAuthority::for_canonical_physical_format(binary);
    let mut bytes = Vec::with_capacity(usize::from(PHYSICAL_HEADER_LENGTH) + payload.len());
    bytes.extend_from_slice(&headers.encode_page_header(
        cell,
        PhysicalPageKind::DataPage,
        payload.len().try_into().expect("bounded fixture payload"),
    ));
    bytes.extend_from_slice(payload);
    std::fs::write(target, bytes).expect("owner-bound page bytes");
}

fn physical_readiness() -> AcceptedHandoffReadiness {
    let digest = |name: &str| StableDigest::new(format!("sha256:{name}")).expect("fixture digest");
    AcceptedHandoffReadiness::from_foundational_handoff_artifacts(
        ROADMAP_2_S1_SCOPE,
        HandoffEvidenceDigestSet::new(
            digest("backend"),
            digest("deferred"),
            digest("harness"),
            digest("terms"),
            digest("audit"),
            digest("complexity"),
            digest("provenance"),
        ),
    )
    .expect("physical-format readiness")
}
