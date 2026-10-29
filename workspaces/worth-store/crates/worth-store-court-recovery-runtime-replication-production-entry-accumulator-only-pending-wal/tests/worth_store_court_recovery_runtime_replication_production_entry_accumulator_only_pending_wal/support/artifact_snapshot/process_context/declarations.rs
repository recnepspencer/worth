pub(crate) use {std::collections::BTreeMap, std::collections::BTreeSet};
pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use {worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity};
pub(crate) use {worth_store_physical_format::RecordArtifactFile};
pub(crate) use {worth_store_recovery_runtime::RecoveryCleanupDispositionKind, worth_store_recovery_runtime::RecoveryCleanupEvidence, worth_store_recovery_runtime::RecoveryCleanupTarget};



pub(crate) struct ArtifactSnapshot {
    pub(crate) required_preexisting: BTreeSet<PathBuf>,
    pub(crate) all_preexisting: BTreeSet<PathBuf>,
    pub(crate) recovery_created: BTreeSet<PathBuf>,
}

impl ArtifactSnapshot {
    pub(crate) fn capture(root: &Path) -> Self {
        let mut required_preexisting = std::collections::BTreeSet::new();
        crate::artifact_snapshot::process_context::selected_records::capture(root, &mut required_preexisting);
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
            recovery_created: std::collections::BTreeSet::new(),
        }
    }

    pub(crate) fn include_recovery_created(
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

    pub(crate) fn assert_reconciled(&self, root: &Path, evidence: &RecoveryCleanupEvidence) {
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
                worth_store_recovery_runtime::RecoveryCleanupDispositionKind::SafelyRemoved => assert!(
                    !exists,
                    "safely removed artifact still exists: {}",
                    path.display()
                ),
                worth_store_recovery_runtime::RecoveryCleanupDispositionKind::Current
                | worth_store_recovery_runtime::RecoveryCleanupDispositionKind::Retained
                | worth_store_recovery_runtime::RecoveryCleanupDispositionKind::Deferred(_)
                | worth_store_recovery_runtime::RecoveryCleanupDispositionKind::QuarantinedOrUnsupported => assert!(
                    exists,
                    "retained cleanup artifact is missing: {}",
                    path.display()
                ),
                worth_store_recovery_runtime::RecoveryCleanupDispositionKind::Eligible => {
                    panic!("terminal cleanup evidence retained Eligible")
                }
            }
        }
    }
}

pub(crate) fn surviving_artifacts(root: &Path) -> BTreeSet<PathBuf> {
    let mut paths = std::collections::BTreeSet::new();
    collect_files(&root.join("families/records"), &mut paths);
    collect_file(&root.join("families/checkpoint.current"), &mut paths);
    collect_files(&root.join("families/wal"), &mut paths);
    collect_files(&root.join("staging/records"), &mut paths);
    paths
}

pub(crate) fn disposition_paths(
    root: &Path,
    evidence: &RecoveryCleanupEvidence,
) -> BTreeMap<PathBuf, RecoveryCleanupDispositionKind> {
    let mut paths = std::collections::BTreeMap::new();
    for disposition in evidence.dispositions() {
        let path = match disposition.target() {
            worth_store_recovery_runtime::RecoveryCleanupTarget::Record(artifact) => record_artifact_path(root, *artifact),
            worth_store_recovery_runtime::RecoveryCleanupTarget::Wal(artifact) => {
                root.join("families/wal").join(artifact.file_name())
            }
            worth_store_recovery_runtime::RecoveryCleanupTarget::Checkpoint(_) => root.join("families/checkpoint.current"),
            worth_store_recovery_runtime::RecoveryCleanupTarget::Residue { name, .. } => {
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

pub(crate) fn missing_preexisting_path(
    snapshot: &BTreeSet<PathBuf>,
    dispositions: &BTreeMap<PathBuf, RecoveryCleanupDispositionKind>,
) -> Option<PathBuf> {
    snapshot
        .iter()
        .find(|path| !dispositions.contains_key(*path))
        .cloned()
}

pub(crate) fn missing_surviving_path(
    snapshot: &BTreeSet<PathBuf>,
    dispositions: &BTreeMap<PathBuf, RecoveryCleanupDispositionKind>,
) -> Option<PathBuf> {
    snapshot
        .iter()
        .find(|path| !dispositions.contains_key(*path))
        .cloned()
}

pub(crate) fn missing_created_path(
    snapshot: &BTreeSet<PathBuf>,
    dispositions: &BTreeMap<PathBuf, RecoveryCleanupDispositionKind>,
) -> Option<PathBuf> {
    snapshot
        .iter()
        .find(|path| !dispositions.contains_key(*path))
        .cloned()
}

pub(crate) fn collect_files(directory: &Path, paths: &mut BTreeSet<PathBuf>) {
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

pub(crate) fn collect_file(path: &Path, paths: &mut BTreeSet<PathBuf>) {
    if path.is_file() {
        paths.insert(path.to_path_buf());
    }
}

pub(crate) fn record_artifact_path(root: &Path, artifact: RecordArtifactFile) -> PathBuf {
    let records = root.join("families/records");
    let directory = match artifact {
        worth_store_physical_format::RecordArtifactFile::BootstrapCatalog
        | worth_store_physical_format::RecordArtifactFile::CurrentRootSelector
        | worth_store_physical_format::RecordArtifactFile::PreviousRootSelector => records,
        worth_store_physical_format::RecordArtifactFile::RootSelectorCandidate { .. }
        | worth_store_physical_format::RecordArtifactFile::CatalogCandidate { .. } => root.join("staging/records"),
        worth_store_physical_format::RecordArtifactFile::RootManifest { .. }
        | worth_store_physical_format::RecordArtifactFile::RootRoutingBlock { .. }
        | worth_store_physical_format::RecordArtifactFile::ReleaseCustodyHeadBlock { .. } => records.join("roots"),
        worth_store_physical_format::RecordArtifactFile::Segment { .. } => records.join("segments"),
        worth_store_physical_format::RecordArtifactFile::SegmentManifest { .. }
        | worth_store_physical_format::RecordArtifactFile::SegmentMembershipBlock { .. } => records.join("segment-manifests"),
        worth_store_physical_format::RecordArtifactFile::ExtentArena { .. } => records.join("arenas"),
        worth_store_physical_format::RecordArtifactFile::FreeSpaceManifest { .. }
        | worth_store_physical_format::RecordArtifactFile::FreeSpaceMembershipBlock { .. } => records.join("free-space"),
    };
    directory.join(artifact.file_name())
}