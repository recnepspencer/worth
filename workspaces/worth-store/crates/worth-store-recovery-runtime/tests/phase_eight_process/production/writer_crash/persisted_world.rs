use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn raw_media_snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn collect(root: &Path, directory: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in std::fs::read_dir(directory).expect("snapshot persisted media") {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(root, &path, files);
            } else if path.extension().is_none_or(|extension| extension != "lock") {
                files.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(path).expect("read persisted media byte"),
                ));
            }
        }
    }
    let mut files = Vec::new();
    collect(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

pub(super) fn changed_paths(
    before: &[(PathBuf, Vec<u8>)],
    after: &[(PathBuf, Vec<u8>)],
) -> Vec<PathBuf> {
    before
        .iter()
        .chain(after.iter())
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| {
            before
                .iter()
                .find(|(found, _)| found == path)
                .map(|(_, bytes)| bytes)
                != after
                    .iter()
                    .find(|(found, _)| found == path)
                    .map(|(_, bytes)| bytes)
        })
        .collect()
}

pub(super) fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("create persisted world copy");
    for entry in std::fs::read_dir(source).expect("enumerate source root") {
        let entry = entry.expect("source root entry");
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy persisted artifact");
        }
    }
}
