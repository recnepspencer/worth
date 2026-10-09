//! No release-head checkpoint can enter Serving through an ordinary reopen.

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::super::shared_format;
use crate::fixture::assert_released_open_requires_recovered_custody;

pub(super) fn assert_open_requires_c8_custody(root: &Path) {
    let before = selected_artifacts(root);
    assert_released_open_requires_recovered_custody(root, shared_format());
    assert_eq!(
        selected_artifacts(root),
        before,
        "Serving denial must not publish a root, free map, WAL member, or payload",
    );
}

fn selected_artifacts(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let families = root.join("families");
    let records = families.join("records");
    let mut artifacts = Vec::new();
    for directory in [
        records.clone(),
        records.join("roots"),
        records.join("free-space"),
        records.join("arenas"),
        records.join("segments"),
        records.join("segment-manifests"),
        families.join("wal"),
    ] {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                artifacts.push((entry.path(), fs::read(entry.path()).unwrap()));
            }
        }
    }
    let checkpoint = families.join("checkpoint.current");
    artifacts.push((checkpoint.clone(), fs::read(checkpoint).unwrap()));
    artifacts.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    artifacts
}
