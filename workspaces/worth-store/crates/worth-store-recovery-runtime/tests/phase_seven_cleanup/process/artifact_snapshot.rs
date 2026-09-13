use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity;
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_runtime::{
    RecoveryCleanupDispositionKind, RecoveryCleanupEvidence, RecoveryCleanupTarget,
};

#[path = "artifact_snapshot/selected_records.rs"]
mod selected_records;

pub(super) struct ArtifactSnapshot {
    required_preexisting: BTreeSet<PathBuf>,
    all_preexisting: BTreeSet<PathBuf>,
    recovery_created: BTreeSet<PathBuf>,
}

impl ArtifactSnapshot {
    pub(super) fn capture(root: &Path) -> Self {
        let mut required_preexisting = BTreeSet::new();
        selected_records::capture(root, &mut required_preexisting);
        collect_file(
            &root.join("families/checkpoint.current"),
            &mut required_preexisting,
        );
        collect_files(&root.join("families/wal"), &mut required_preexisting);
        let all_preexisting = surviving_artifacts(root);
        assert!(required_preexisting.is_subset(&all_preexisting));
        Self {
            required_preexisting,
            all_preexisting,
            recovery_created: BTreeSet::new(),
        }
    }

    pub(super) fn include_recovery_created(
        &mut self,
        root: &Path,
        artifacts: &[RecordArtifactFile],
    ) {
        self.recovery_created.extend(
            artifacts
                .iter()
                .map(|artifact| record_artifact_path(root, *artifact)),
        );
    }

    pub(super) fn assert_reconciled(&self, root: &Path, evidence: &RecoveryCleanupEvidence) {
        let dispositions = disposition_paths(root, evidence);
        if let Some(path) = missing_preexisting_path(&self.required_preexisting, &dispositions) {
            panic!(
                "pre-recovery artifact has no cleanup disposition: {}; dispositions: {:?}",
                path.display(),
                dispositions.keys().collect::<Vec<_>>()
            );
        }
        let created_and_surviving = surviving_artifacts(root)
            .difference(&self.all_preexisting)
            .cloned()
            .collect::<BTreeSet<_>>();
        if let Some(path) = missing_surviving_path(&created_and_surviving, &dispositions) {
            panic!(
                "post-recovery artifact has no cleanup disposition: {}; dispositions: {:?}",
                path.display(),
                dispositions.keys().collect::<Vec<_>>()
            );
        }
        if let Some(path) = missing_created_path(&self.recovery_created, &dispositions) {
            panic!(
                "recovery-created artifact has no cleanup disposition: {}; dispositions: {:?}",
                path.display(),
                dispositions.keys().collect::<Vec<_>>()
            );
        }
        for (path, kind) in dispositions {
            assert!(
                self.all_preexisting.contains(&path) || self.recovery_created.contains(&path),
                "cleanup disposition names an artifact outside preexisting and recovery-created truth: {}",
                path.display(),
            );
            let exists = path.exists();
            match kind {
                RecoveryCleanupDispositionKind::SafelyRemoved => assert!(
                    !exists,
                    "safely removed artifact still exists: {}",
                    path.display()
                ),
                RecoveryCleanupDispositionKind::Current
                | RecoveryCleanupDispositionKind::Retained
                | RecoveryCleanupDispositionKind::Deferred(_)
                | RecoveryCleanupDispositionKind::QuarantinedOrUnsupported => assert!(
                    exists,
                    "retained cleanup artifact is missing: {}",
                    path.display()
                ),
                RecoveryCleanupDispositionKind::Eligible => {
                    panic!("terminal cleanup evidence retained Eligible")
                }
            }
        }
    }
}

fn surviving_artifacts(root: &Path) -> BTreeSet<PathBuf> {
    let mut paths = BTreeSet::new();
    collect_files(&root.join("families/records"), &mut paths);
    collect_file(&root.join("families/checkpoint.current"), &mut paths);
    collect_files(&root.join("families/wal"), &mut paths);
    collect_files(&root.join("staging/records"), &mut paths);
    paths
}

fn disposition_paths(
    root: &Path,
    evidence: &RecoveryCleanupEvidence,
) -> BTreeMap<PathBuf, RecoveryCleanupDispositionKind> {
    let mut paths = BTreeMap::new();
    for disposition in evidence.dispositions() {
        let path = match disposition.target() {
            RecoveryCleanupTarget::Record(artifact) => record_artifact_path(root, *artifact),
            RecoveryCleanupTarget::Wal(artifact) => {
                root.join("families/wal").join(artifact.file_name())
            }
            RecoveryCleanupTarget::Checkpoint(_) => root.join("families/checkpoint.current"),
            RecoveryCleanupTarget::Residue { name, .. } => {
                root.join("families/wal").join(name.as_ref())
            }
        };
        assert!(
            paths.insert(path, disposition.kind()).is_none(),
            "duplicate cleanup disposition path"
        );
    }
    paths
}

fn missing_preexisting_path(
    snapshot: &BTreeSet<PathBuf>,
    dispositions: &BTreeMap<PathBuf, RecoveryCleanupDispositionKind>,
) -> Option<PathBuf> {
    snapshot
        .iter()
        .find(|path| !dispositions.contains_key(*path))
        .cloned()
}

fn missing_surviving_path(
    snapshot: &BTreeSet<PathBuf>,
    dispositions: &BTreeMap<PathBuf, RecoveryCleanupDispositionKind>,
) -> Option<PathBuf> {
    snapshot
        .iter()
        .find(|path| !dispositions.contains_key(*path))
        .cloned()
}

fn missing_created_path(
    snapshot: &BTreeSet<PathBuf>,
    dispositions: &BTreeMap<PathBuf, RecoveryCleanupDispositionKind>,
) -> Option<PathBuf> {
    snapshot
        .iter()
        .find(|path| !dispositions.contains_key(*path))
        .cloned()
}

fn collect_files(directory: &Path, paths: &mut BTreeSet<PathBuf>) {
    if !directory.exists() {
        return;
    }
    for entry in std::fs::read_dir(directory).expect("enumerate persisted artifacts") {
        let path = entry.expect("persisted artifact entry").path();
        if path.is_dir() {
            collect_files(&path, paths);
        } else if path.is_file() {
            paths.insert(path);
        }
    }
}

fn collect_file(path: &Path, paths: &mut BTreeSet<PathBuf>) {
    if path.is_file() {
        paths.insert(path.to_path_buf());
    }
}

fn record_artifact_path(root: &Path, artifact: RecordArtifactFile) -> PathBuf {
    let records = root.join("families/records");
    let directory = match artifact {
        RecordArtifactFile::BootstrapCatalog
        | RecordArtifactFile::CurrentRootSelector
        | RecordArtifactFile::PreviousRootSelector => records,
        RecordArtifactFile::RootSelectorCandidate { .. }
        | RecordArtifactFile::CatalogCandidate { .. } => root.join("staging/records"),
        RecordArtifactFile::RootManifest { .. } | RecordArtifactFile::RootRoutingBlock { .. } => {
            records.join("roots")
        }
        RecordArtifactFile::Segment { .. } => records.join("segments"),
        RecordArtifactFile::SegmentManifest { .. }
        | RecordArtifactFile::SegmentMembershipBlock { .. } => records.join("segment-manifests"),
        RecordArtifactFile::Extent { .. } => records.join("extents"),
        RecordArtifactFile::ExtentManifest { .. } => records.join("extent-manifests"),
        RecordArtifactFile::FreeSpaceManifest { .. }
        | RecordArtifactFile::FreeSpaceMembershipBlock { .. } => records.join("free-space"),
    };
    directory.join(artifact.file_name())
}

#[test]
fn an_omitted_preexisting_artifact_cannot_be_hidden_by_the_cleanup_oracle() {
    let path = PathBuf::from("families/wal").join(
        WalSegmentArtifactIdentity::parse("segment-1-generation-1.wal")
            .unwrap()
            .file_name(),
    );
    let snapshot = BTreeSet::from([path.clone()]);
    assert_eq!(
        missing_preexisting_path(&snapshot, &BTreeMap::new()),
        Some(path)
    );
}

#[test]
fn an_omitted_recovery_created_artifact_cannot_be_hidden_by_the_cleanup_oracle() {
    let path = PathBuf::from("families/records/segments/segment-9-generation-2.data");
    assert_eq!(
        missing_created_path(&BTreeSet::from([path.clone()]), &BTreeMap::new()),
        Some(path)
    );
}
