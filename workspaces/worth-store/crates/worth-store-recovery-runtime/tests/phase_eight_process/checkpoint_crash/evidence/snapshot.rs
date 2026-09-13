use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use sha2::{Digest, Sha256};

pub(crate) type DirectorySnapshot = BTreeMap<String, (u64, [u8; 32])>;

pub(crate) fn snapshot_directory(root: &Path) -> DirectorySnapshot {
    let mut snapshot = BTreeMap::new();
    collect_snapshot(root, root, &mut snapshot);
    snapshot
}

pub(crate) fn assert_snapshot_preserved(
    root: &Path,
    baseline: &DirectorySnapshot,
    effect: &DirectorySnapshot,
    stage: &str,
) {
    let changed_paths = baseline
        .keys()
        .chain(effect.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| baseline.get(path) != effect.get(path))
        .collect::<BTreeSet<_>>();
    let current = snapshot_directory(root);
    let paths = baseline
        .keys()
        .chain(effect.keys())
        .chain(current.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for path in paths {
        let expected = if changed_paths.contains(&path) {
            effect.get(&path)
        } else {
            baseline.get(&path)
        };
        assert_eq!(
            current.get(&path),
            expected,
            "checkpoint crash changed an unmodeled {stage} frontier artifact {path}"
        );
    }
}

pub(crate) fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("create recovery copy root");
    for entry in std::fs::read_dir(source).expect("read recovery copy source") {
        let entry = entry.expect("read recovery copy entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path).expect("copy recovery source artifact");
        }
    }
}

fn collect_snapshot(root: &Path, current: &Path, snapshot: &mut DirectorySnapshot) {
    for entry in std::fs::read_dir(current).expect("read snapshot directory") {
        let entry = entry.expect("read snapshot entry");
        let path = entry.path();
        if path.is_dir() {
            collect_snapshot(root, &path, snapshot);
        } else if path
            .file_name()
            .is_none_or(|name| !name.to_string_lossy().ends_with(".lock"))
        {
            let bytes = std::fs::read(&path).expect("read snapshot artifact");
            let digest: [u8; 32] = Sha256::digest(&bytes).into();
            let relative = path
                .strip_prefix(root)
                .expect("snapshot path under root")
                .to_string_lossy()
                .replace('\\', "/");
            snapshot.insert(relative, (bytes.len() as u64, digest));
        }
    }
}
