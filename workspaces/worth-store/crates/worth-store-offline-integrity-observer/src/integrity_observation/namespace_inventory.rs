use super::{
    unknown_artifact::{relative_path, unknown_artifact},
    BoundedMediaWalk, OfflineArtifactObservation,
};
use std::collections::{BTreeSet, VecDeque};
use std::path::Path;

/// Enumerates namespace residue within the same walk bounds; it never probes unknown bytes.
pub(crate) fn observe_namespace_residue(
    root: &Path,
    observed: &[OfflineArtifactObservation],
    walk: &mut BoundedMediaWalk,
) -> Vec<OfflineArtifactObservation> {
    if walk.exhausted_reason().is_some() {
        return Vec::new();
    }
    let covered: BTreeSet<_> = observed
        .iter()
        .map(|artifact| artifact.relative_path().to_owned())
        .collect();
    let mut queue = VecDeque::from([(root.to_owned(), 0)]);
    let mut residue = Vec::new();
    while let Some((directory, depth)) = queue.pop_front() {
        let scan = match walk.scan_directory(&directory, depth) {
            Ok(scan) => scan,
            Err(_) => {
                residue.push(incomplete_directory(
                    root,
                    &directory,
                    depth,
                    super::OfflineIndeterminatePhysicalReason::IoFailure,
                    walk,
                ));
                continue;
            }
        };
        if let Some(reason) = scan.incomplete_reason {
            residue.push(incomplete_directory(root, &directory, depth, reason, walk));
        }
        for path in scan.entries {
            let relative = relative_path(root, &path);
            if covered.contains(&relative) {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&path);
            let is_directory = metadata
                .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink());
            if is_directory {
                if !canonical_directory(&relative) {
                    residue.push(unknown_artifact(root, &path, depth + 1, walk));
                }
                if queue.len() as u64 >= walk.maximum_entries() {
                    walk.entry_bound();
                    break;
                }
                queue.push_back((path, depth + 1));
            } else {
                residue.push(unknown_artifact(root, &path, depth + 1, walk));
            }
        }
    }
    residue
}

fn incomplete_directory(
    root: &Path,
    path: &Path,
    depth: u32,
    reason: super::OfflineIndeterminatePhysicalReason,
    walk: &mut BoundedMediaWalk,
) -> OfflineArtifactObservation {
    let observation = unknown_artifact(root, path, depth, walk);
    let outcome = super::OfflineIntegrityOutcome::Indeterminate(reason);
    walk.record_outcome(&outcome);
    observation.with_outcome(outcome)
}

fn canonical_directory(path: &str) -> bool {
    matches!(
        path,
        "namespace"
            | "families"
            | "families/records"
            | "families/records/roots"
            | "families/records/segments"
            | "families/records/segment-manifests"
            | "families/records/extents"
            | "families/records/extent-manifests"
            | "families/records/free-space"
            | "families/physical-work"
            | "families/wal"
            | "staging"
            | "staging/records"
    )
}
