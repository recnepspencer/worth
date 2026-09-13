use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_blob_chunks::certification_test_authority::blob_backup_artifact_for_bytes;
use worth_store_layout_indexes::encode_baseline_btree_leaf_record;
use worth_store_offline_verifier::checkpoint_backup_frontier_digest;
use worth_store_physical_backend::observe_physical_backup_artifact;
use worth_store_physical_format::{
    BackupBundleArtifactFormat, CheckpointBackupArtifact, CheckpointBackupArtifactInput,
    InMemoryPhysicalFormatModel, PageGenerationCell, PersistedExtentBytes, PersistedPageBytes,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalReferenceAuthority,
    PlatformPhysicalRootPublicationReport, RootPublicationCell,
};
use worth_store_physical_isolation::{
    BackupArtifactCoverage, BackupArtifactFamily, BackupArtifactReference,
    CurrentGenerationPhysicalReference, UntrustedBackupArtifactClaim,
};

use super::physical_reference::{
    current_extent_reference, current_page_reference, current_root_reference,
    current_segment_reference, current_slot_reference, extent, extent_identity, page,
    page_identity, record_slot, root_identity, segment,
};
use super::CanonicalBackupArtifacts;

pub(super) struct CanonicalBackupArtifactRequest<'a> {
    pub(super) case: &'a str,
    pub(super) source: &'a Path,
    pub(super) runtime: &'a InMemoryPhysicalFormatModel,
    pub(super) publication: PlatformPhysicalRootPublicationReport,
    pub(super) generation: PhysicalGeneration,
    pub(super) blob_count: u64,
}

pub(super) fn materialize_canonical_backup_artifacts(
    request: CanonicalBackupArtifactRequest<'_>,
) -> CanonicalBackupArtifacts {
    let CanonicalBackupArtifactRequest {
        case,
        source,
        runtime,
        publication,
        generation,
        blob_count,
    } = request;
    let current = runtime
        .current_physical_reachability_source()
        .expect("published current-root closure");
    let root = current.manifest().root_publication();
    let layout = publication.persisted_layout();
    let persisted_page = layout.pages().first().expect("persisted page");
    let persisted_extent = layout.extents().first().expect("persisted extent");
    let mut references = Vec::with_capacity(6 + blob_count as usize);
    references.push(root_artifact(
        source,
        layout
            .root_manifest_candidates()
            .first()
            .expect("root manifest"),
        root,
    ));
    let (checkpoint, checkpoint_identity) =
        checkpoint_artifact(source, runtime, root, persisted_page.cell(), generation);
    references.push(checkpoint);
    references.push(wal_artifact(case, source, generation));
    references.push(page_artifact(source, persisted_page));
    references.push(extent_artifact(source, persisted_extent));
    references.push(index_artifact(source, generation));
    references.extend(blob_artifacts(case, source, generation, blob_count));

    CanonicalBackupArtifacts {
        references,
        checkpoint_identity,
    }
}

fn root_artifact(
    source: &Path,
    bytes: &[u8],
    root: RootPublicationCell,
) -> BackupArtifactReference {
    observe_reference(
        source,
        FixtureArtifactMedia::new("root.media", bytes),
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::RootManifest,
            format: BackupBundleArtifactFormat::PhysicalRootManifestV1,
            identity: root_identity(root),
            generation: root.generation().get(),
            coverage: BackupArtifactCoverage::root_manifest(root.generation().get())
                .expect("root coverage"),
        },
        current_root_reference(root),
    )
}

fn checkpoint_artifact(
    source: &Path,
    runtime: &InMemoryPhysicalFormatModel,
    root: RootPublicationCell,
    checkpoint_page: PageGenerationCell,
    generation: PhysicalGeneration,
) -> (BackupArtifactReference, String) {
    let checkpoint_identity = format!(
        "checkpoint:{}:{}",
        generation.get(),
        root.generation().get()
    );
    let manifest_generation = generation.get();
    let durable_checkpoint_lsn = 10;
    let covered_lsn = (10, 12);
    let redo_lsn = 10;
    let pages = vec![(checkpoint_page, redo_lsn)];
    let root_reference = PhysicalReferenceAuthority::for_canonical_physical_format()
        .admit_root_publication(root)
        .reference();
    let checkpoint = CheckpointBackupArtifact::from_input(CheckpointBackupArtifactInput {
        checkpoint_identity: checkpoint_identity.clone(),
        manifest_generation,
        durable_checkpoint_lsn,
        root: root_reference,
        covered_lsn,
        redo_lsn,
        pages: pages.clone(),
    })
    .expect("sharp checkpoint backup artifact");
    let checkpoint_identity = checkpoint.checkpoint_identity().to_owned();
    let authority_fingerprint = runtime.store_identity().authority_identity().fingerprint();
    let frontier_digest = checkpoint_backup_frontier_digest(
        authority_fingerprint,
        &checkpoint_identity,
        manifest_generation,
        durable_checkpoint_lsn,
        root,
        covered_lsn,
        redo_lsn,
        &pages,
    );
    let mut bytes = Vec::new();
    checkpoint.encode(&mut bytes).expect("checkpoint encoding");
    let reference = current_slot_reference(segment(200), page(1), record_slot(2), generation);
    let artifact = observe_reference(
        source,
        FixtureArtifactMedia::new("checkpoint.media", &bytes),
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::CheckpointManifest,
            format: BackupBundleArtifactFormat::RecoveryCheckpointManifestV1,
            identity: checkpoint_identity.clone(),
            generation: reference.generation().get(),
            coverage: BackupArtifactCoverage::checkpoint_manifest(
                &checkpoint_identity,
                manifest_generation,
                durable_checkpoint_lsn,
                authority_fingerprint,
                frontier_digest,
            )
            .expect("checkpoint coverage"),
        },
        reference,
    );
    (artifact, checkpoint_identity)
}

fn wal_artifact(
    case: &str,
    source: &Path,
    generation: PhysicalGeneration,
) -> BackupArtifactReference {
    let owner_segment = segment(3);
    let wal = worth_store_wal::prepare_wal_frame_append(
        source,
        owner_segment.get(),
        generation.get(),
        10,
        12,
        &format!("{case}:wal-frame"),
        format!("{case}:wal-payload").as_bytes(),
    )
    .expect("canonical WAL frame");
    observe_reference(
        source,
        FixtureArtifactMedia::new("wal.media", wal.encoded_frame()),
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::WalSegment,
            format: BackupBundleArtifactFormat::WalSegmentV1,
            identity: format!("wal:{}:{}:10-12", owner_segment.get(), generation.get()),
            generation: generation.get(),
            coverage: BackupArtifactCoverage::wal_segment(10, 12).expect("WAL coverage"),
        },
        current_segment_reference(owner_segment, generation),
    )
}

fn page_artifact(source: &Path, persisted: &PersistedPageBytes) -> BackupArtifactReference {
    let reference = current_page_reference(persisted.cell());
    observe_reference(
        source,
        FixtureArtifactMedia::new("page.media", persisted.bytes()),
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::Page,
            format: BackupBundleArtifactFormat::PhysicalDataPageV1,
            identity: page_identity(persisted.cell()),
            generation: reference.generation().get(),
            coverage: BackupArtifactCoverage::physical_reachability(),
        },
        reference,
    )
}

fn extent_artifact(source: &Path, persisted: &PersistedExtentBytes) -> BackupArtifactReference {
    let reference = current_extent_reference(persisted.cell());
    observe_reference(
        source,
        FixtureArtifactMedia::new("extent.media", persisted.bytes()),
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::Extent,
            format: BackupBundleArtifactFormat::PhysicalExtentRecordV1,
            identity: extent_identity(persisted.cell()),
            generation: reference.generation().get(),
            coverage: BackupArtifactCoverage::physical_reachability(),
        },
        reference,
    )
}

fn index_artifact(source: &Path, generation: PhysicalGeneration) -> BackupArtifactReference {
    let bytes = encode_baseline_btree_leaf_record([record_slot(20), record_slot(21)], true, false);
    let reference = current_slot_reference(segment(200), page(1), record_slot(6), generation);
    observe_reference(
        source,
        FixtureArtifactMedia::new("index.media", &bytes),
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::Index,
            format: BackupBundleArtifactFormat::LayoutBTreeLeafV1,
            identity: format!("index:sha256:{}", hex(&Sha256::digest(bytes))),
            generation: reference.generation().get(),
            coverage: BackupArtifactCoverage::physical_reachability(),
        },
        reference,
    )
}

fn blob_artifacts(
    case: &str,
    source: &Path,
    generation: PhysicalGeneration,
    blob_count: u64,
) -> Vec<BackupArtifactReference> {
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    (0..blob_count)
        .map(|index| {
            let blob_case = format!("{case}:blob-{index:04}");
            let blob_payload = format!("{blob_case}:payload");
            let blob = blob_backup_artifact_for_bytes(&blob_case, blob_payload.as_bytes());
            let mut bytes = Vec::new();
            blob.encode(&mut bytes).expect("blob backup encoding");
            let reference = current_extent_reference(
                generations
                    .extent_cell(segment(100), extent(7 + index))
                    .with_extent_generation(generation),
            );
            observe_reference(
                source,
                FixtureArtifactMedia::new(&format!("blob-{index:04}.media"), &bytes),
                UntrustedBackupArtifactClaim {
                    family: BackupArtifactFamily::BlobChunk,
                    format: BackupBundleArtifactFormat::BlobChunkV1,
                    identity: blob.chunk_identity().to_owned(),
                    generation: reference.generation().get(),
                    coverage: BackupArtifactCoverage::physical_reachability(),
                },
                reference,
            )
        })
        .collect()
}

struct FixtureArtifactMedia<'a> {
    name: &'a str,
    bytes: &'a [u8],
}

impl<'a> FixtureArtifactMedia<'a> {
    const fn new(name: &'a str, bytes: &'a [u8]) -> Self {
        Self { name, bytes }
    }
}

fn observe_reference(
    source: &Path,
    media: FixtureArtifactMedia<'_>,
    claim: UntrustedBackupArtifactClaim,
    reclaim_reference: CurrentGenerationPhysicalReference,
) -> BackupArtifactReference {
    let path = source.join(media.name);
    std::fs::write(&path, media.bytes).expect("owner artifact bytes");
    BackupArtifactReference::declare_untrusted_physical_observation(
        claim,
        observe_physical_backup_artifact(path, 4 * 1024).expect("physical observation"),
        reclaim_reference,
    )
    .expect("owner-bound backup reference")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
