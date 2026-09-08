use super::{ProcessManifestDenial, ProcessStoreFile};
use std::{collections::{BTreeMap, BTreeSet},path::{Path,PathBuf}};
use sha2::{Digest,Sha256};

pub(super) fn observe_tree(
    root: &Path,
    live_lease_payload_excluded: bool,
) -> Result<
    (
        BTreeSet<PathBuf>,
        BTreeMap<PathBuf, ProcessStoreFile>,
        BTreeMap<PathBuf, Vec<u8>>,
    ),
    ProcessManifestDenial,
> {
    let mut observed_directories = BTreeSet::new();
    let mut files = BTreeMap::new();
    let mut contents = BTreeMap::new();
    let mut directories = vec![PathBuf::new()];
    while let Some(relative_directory) = directories.pop() {
        let mut entries = std::fs::read_dir(root.join(&relative_directory))
            .map_err(|_| ProcessManifestDenial::TreeRead)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ProcessManifestDenial::TreeRead)?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries.into_iter().rev() {
            let relative = relative_directory.join(entry.file_name());
            let kind = entry
                .file_type()
                .map_err(|_| ProcessManifestDenial::TreeRead)?;
            if kind.is_dir() {
                observed_directories.insert(relative.clone());
                directories.push(relative);
            } else if kind.is_file() {
                // An ordinary runtime owns this non-artifact lease exclusively on Windows.
                // Its presence is required above; its payload is explicitly outside a live
                // diagnostic snapshot, never replaced with claimed observed bytes.
                if live_lease_payload_excluded && relative == Path::new("namespace/mutation.lock") {
                    continue;
                }
                let bytes =
                    std::fs::read(entry.path()).map_err(|_| ProcessManifestDenial::TreeRead)?;
                files.insert(
                    relative.clone(),
                    ProcessStoreFile {
                        exact_length: bytes.len() as u64,
                        content_sha256: Sha256::digest(&bytes).into(),
                    },
                );
                contents.insert(relative, bytes);
            } else {
                return Err(ProcessManifestDenial::NonRegularEntry(relative));
            }
        }
    }
    Ok((observed_directories, files, contents))
}

