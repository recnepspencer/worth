use std::path::Path;

use sha2::{Digest, Sha256};

use super::{ARTIFACT_IDENTITY_DOMAIN, ARTIFACT_SET_DOMAIN};

pub(super) fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    let mut entries = std::fs::read_dir(directory)
        .map_err(|error| format!("read parent history directory {directory:?}: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read parent history entry: {error}"))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if entry
            .file_type()
            .map_err(|error| format!("read parent history file type: {error}"))?
            .is_dir()
        {
            collect_files(root, &path, files)?;
        } else if path.is_file() {
            if path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".lock"))
            {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|error| format!("relative parent history path: {error}"))?
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = std::fs::read(&path)
                .map_err(|error| format!("read parent history artifact {path:?}: {error}"))?;
            files.push((relative, bytes));
        }
    }
    Ok(())
}

pub(super) fn artifact_set_digest(contents: &[(String, u64, [u8; 32])]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(ARTIFACT_SET_DOMAIN);
    digest.update((contents.len() as u64).to_le_bytes());
    for (path, bytes, content_digest) in contents {
        let mut identity = Vec::with_capacity(8 + path.len() + 8 + 32);
        identity.extend_from_slice(&(path.len() as u64).to_le_bytes());
        identity.extend_from_slice(path.as_bytes());
        identity.extend_from_slice(&bytes.to_le_bytes());
        identity.extend_from_slice(content_digest);
        digest.update(&identity);
    }
    digest.finalize().into()
}

pub(super) fn artifact_identity_digest(contents: &[(String, u64, [u8; 32])]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(ARTIFACT_IDENTITY_DOMAIN);
    for (path, bytes, content_digest) in contents {
        let mut identity = Vec::with_capacity(8 + path.len() + 8 + 32);
        identity.extend_from_slice(&(path.len() as u64).to_le_bytes());
        identity.extend_from_slice(path.as_bytes());
        identity.extend_from_slice(&bytes.to_le_bytes());
        identity.extend_from_slice(content_digest);
        digest.update((identity.len() as u64).to_le_bytes());
        digest.update(identity);
    }
    digest.update((contents.len() as u64).to_le_bytes());
    digest.finalize().into()
}
