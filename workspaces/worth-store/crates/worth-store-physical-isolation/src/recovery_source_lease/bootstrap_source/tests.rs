use std::path::PathBuf;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    OfflineMediaClosureEntry, OfflineMediaConsistencyBasis, ReadOnlyOfflineMediaCapability,
};

use super::{
    BootstrapSourceArtifact, BootstrapSourceArtifactFamily, BootstrapSourceEvidenceBinding,
    BootstrapSourceFrontier, BootstrapSourceResolutionRequest,
    PhysicalIsolationBootstrapSourceOwner,
};

#[test]
fn bounded_real_media_resolution_issues_an_exact_isolation_owned_cut() {
    let world = BootstrapSourceWorld::create();
    let cut =
        PhysicalIsolationBootstrapSourceOwner::resolve(world.request(), world.open_media(), 3)
            .expect("complete content-addressed media must resolve");

    assert_eq!(cut.operation_identity(), [1; 32]);
    assert_eq!(cut.source_identity(), [2; 32]);
    assert_eq!(cut.verification_identity(), [3; 32]);
    assert_eq!(cut.source_lineage_identity(), [4; 32]);
    assert_eq!(cut.source_root(), world.root.as_path());
    assert_eq!(cut.artifact_paths(), world.relative_paths.as_slice());
    assert_eq!(cut.counters().artifacts_reopened(), 4);
    assert_eq!(cut.counters().bytes_read(), world.total_bytes);
    assert_eq!(cut.counters().resident_buffer_bytes(), 3);
    assert_ne!(cut.frontier_identity(), [0; 32]);
    assert_ne!(cut.resolution_identity(), [0; 32]);
}

#[test]
fn same_length_source_substitution_after_open_cannot_resolve() {
    let world = BootstrapSourceWorld::create();
    let media = world.open_media();
    std::fs::write(world.root.join(&world.relative_paths[2]), b"EVIL")
        .expect("mutant write must land");

    let denial = PhysicalIsolationBootstrapSourceOwner::resolve(world.request(), media, 2)
        .expect_err("changed bytes must not become a resolved cut");
    assert!(matches!(
        denial,
        super::BootstrapSourceResolutionDenial::ArtifactDigestMismatch
            | super::BootstrapSourceResolutionDenial::Media(
                worth_store_physical_backend::OfflineMediaReadDenial::ConcurrentMutationIndeterminate { .. }
            )
    ));
}

#[test]
fn a_digest_complete_set_without_blob_reachability_is_rejected() {
    let world = BootstrapSourceWorld::create();
    let artifacts = world
        .artifacts
        .iter()
        .filter(|artifact| artifact.family() != BootstrapSourceArtifactFamily::Blob)
        .cloned()
        .collect();
    let denial = BootstrapSourceResolutionRequest::from_independent_verification(
        [1; 32],
        evidence(),
        &world.root,
        frontier(),
        artifacts,
    )
    .expect_err("blob-incomplete media must fail before physical resolution");

    assert!(matches!(
        denial,
        super::BootstrapSourceResolutionDenial::MissingRequiredFamily(
            BootstrapSourceArtifactFamily::Blob
        )
    ));
}

struct BootstrapSourceWorld {
    _directory: tempfile::TempDir,
    root: PathBuf,
    relative_paths: Vec<PathBuf>,
    artifacts: Vec<BootstrapSourceArtifact>,
    total_bytes: u64,
}

impl BootstrapSourceWorld {
    fn create() -> Self {
        let directory = tempfile::Builder::new()
            .prefix("worth-store-bootstrap-source-")
            .tempdir()
            .expect("bootstrap source directory");
        let root = directory.path().to_owned();
        std::fs::create_dir_all(root.join("blob")).expect("fixture directories must exist");
        let declarations = [
            (
                BootstrapSourceArtifactFamily::Authority,
                "authority.bin",
                b"authority".as_slice(),
            ),
            (
                BootstrapSourceArtifactFamily::Checkpoint,
                "checkpoint.bin",
                b"checkpoint".as_slice(),
            ),
            (
                BootstrapSourceArtifactFamily::Wal,
                "wal.bin",
                b"WAL!".as_slice(),
            ),
            (
                BootstrapSourceArtifactFamily::Blob,
                "blob/chunk.bin",
                b"blob-chunk".as_slice(),
            ),
        ];
        let mut relative_paths = Vec::new();
        let mut artifacts = Vec::new();
        let mut total_bytes = 0_u64;
        for (family, relative, bytes) in declarations {
            let relative = PathBuf::from(relative);
            std::fs::write(root.join(&relative), bytes).expect("fixture bytes must persist");
            total_bytes += bytes.len() as u64;
            artifacts.push(
                BootstrapSourceArtifact::declare(
                    family,
                    &relative,
                    bytes.len() as u64,
                    Sha256::digest(bytes).into(),
                )
                .expect("fixture artifact must be legal"),
            );
            relative_paths.push(relative);
        }
        relative_paths.sort();
        artifacts.sort_by(|left, right| left.relative_path().cmp(right.relative_path()));
        Self {
            _directory: directory,
            root: std::fs::canonicalize(root).expect("fixture root must canonicalize"),
            relative_paths,
            artifacts,
            total_bytes,
        }
    }

    fn request(&self) -> BootstrapSourceResolutionRequest {
        BootstrapSourceResolutionRequest::from_independent_verification(
            [1; 32],
            evidence(),
            &self.root,
            frontier(),
            self.artifacts.clone(),
        )
        .expect("fixture request must be structurally admissible")
    }

    fn open_media(&self) -> ReadOnlyOfflineMediaCapability {
        let entries = self
            .artifacts
            .iter()
            .map(|artifact| {
                OfflineMediaClosureEntry::new(
                    self.root.join(artifact.relative_path()),
                    artifact.byte_length(),
                    artifact.content_digest(),
                )
                .expect("fixture closure entry must be legal")
            })
            .collect::<Vec<_>>();
        let paths = entries
            .iter()
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<_>>();
        let basis = OfflineMediaConsistencyBasis::content_addressed_closure_from_owned_entries(
            "verified-source",
            entries,
        )
        .expect("fixture closure must be admissible");
        ReadOnlyOfflineMediaCapability::open_bounded_from_owned_paths(paths, basis, 16_384)
            .expect("fixture media must open read-only")
    }
}

fn evidence() -> BootstrapSourceEvidenceBinding {
    BootstrapSourceEvidenceBinding::from_independent_verification([2; 32], [3; 32], [4; 32])
        .expect("fixture evidence binding must be legal")
}

fn frontier() -> BootstrapSourceFrontier {
    BootstrapSourceFrontier::admit(19, 18, 17, 16, 7).expect("fixture frontier must be ordered")
}
