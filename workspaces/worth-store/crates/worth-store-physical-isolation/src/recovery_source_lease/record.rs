use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs::OpenOptions, io::Write};

use sha2::{Digest, Sha256};

use super::directory_durability::sync_directory;
use super::{RecoverySourceLeaseDenial, RecoverySourceLeaseKind};

const MAGIC: &[u8; 8] = b"WRSLEAS3";
const VERSION: u16 = 3;
const MAX_RECORD_BYTES: usize = 1024 * 1024;
static NEXT_PENDING_RECORD: AtomicU64 = AtomicU64::new(1);

pub(super) struct PersistedRecoverySourceLease {
    pub(super) kind: RecoverySourceLeaseKind,
    pub(super) operation_identity: [u8; 32],
    pub(super) source_identity: [u8; 32],
    pub(super) source_evidence_identity: [u8; 32],
    pub(super) source_root: PathBuf,
    pub(super) artifact_names: Vec<String>,
}

pub(super) fn encode(
    kind: RecoverySourceLeaseKind,
    operation: [u8; 32],
    source: [u8; 32],
    source_evidence: [u8; 32],
    source_root: &Path,
    artifact_names: &[String],
) -> Result<Vec<u8>, RecoverySourceLeaseDenial> {
    let (platform, root) = encode_path(source_root);
    let root_len =
        u32::try_from(root.len()).map_err(|_| RecoverySourceLeaseDenial::AllocationFailed)?;
    let artifact_count = u32::try_from(artifact_names.len())
        .map_err(|_| RecoverySourceLeaseDenial::AllocationFailed)?;
    let mut content = Vec::new();
    content
        .try_reserve(143 + root.len())
        .map_err(|_| RecoverySourceLeaseDenial::AllocationFailed)?;
    content.extend_from_slice(MAGIC);
    content.extend_from_slice(&VERSION.to_le_bytes());
    content.push(kind.tag());
    content.extend_from_slice(&operation);
    content.extend_from_slice(&source);
    content.extend_from_slice(&source_evidence);
    content.push(platform);
    content.extend_from_slice(&root_len.to_le_bytes());
    content.extend_from_slice(&root);
    content.extend_from_slice(&artifact_count.to_le_bytes());
    for name in artifact_names {
        let bytes = name.as_bytes();
        let length =
            u32::try_from(bytes.len()).map_err(|_| RecoverySourceLeaseDenial::AllocationFailed)?;
        content.extend_from_slice(&length.to_le_bytes());
        content.extend_from_slice(bytes);
    }
    if content.len() > MAX_RECORD_BYTES {
        return Err(RecoverySourceLeaseDenial::RecordTooLarge);
    }
    Ok(content)
}

pub(super) fn decode(
    content: &[u8],
    expected_identity: [u8; 32],
) -> Result<PersistedRecoverySourceLease, RecoverySourceLeaseDenial> {
    if content.len() > MAX_RECORD_BYTES
        || content.len() < 116
        || &content[..8] != MAGIC
        || u16::from_le_bytes([content[8], content[9]]) != VERSION
        || Sha256::digest(content)[..] != expected_identity
    {
        return Err(RecoverySourceLeaseDenial::LeaseConflict);
    }
    let kind = RecoverySourceLeaseKind::from_tag(content[10])?;
    let operation_identity = content[11..43]
        .try_into()
        .expect("fixed operation identity");
    let source_identity = content[43..75].try_into().expect("fixed source identity");
    let source_evidence_identity = content[75..107]
        .try_into()
        .expect("fixed source evidence identity");
    let platform = content[107];
    let root_len = usize::try_from(u32::from_le_bytes(
        content[108..112].try_into().expect("fixed root length"),
    ))
    .map_err(|_| RecoverySourceLeaseDenial::LeaseConflict)?;
    let root_end = 112_usize
        .checked_add(root_len)
        .ok_or(RecoverySourceLeaseDenial::LeaseConflict)?;
    let count_end = root_end
        .checked_add(4)
        .ok_or(RecoverySourceLeaseDenial::LeaseConflict)?;
    let root_bytes = content
        .get(112..root_end)
        .ok_or(RecoverySourceLeaseDenial::LeaseConflict)?;
    let source_root = decode_path(platform, root_bytes)?;
    let count_bytes = content
        .get(root_end..count_end)
        .ok_or(RecoverySourceLeaseDenial::LeaseConflict)?;
    let count = usize::try_from(u32::from_le_bytes(
        count_bytes.try_into().expect("fixed artifact count"),
    ))
    .map_err(|_| RecoverySourceLeaseDenial::LeaseConflict)?;
    let (artifact_names, position) = decode_names(content, count_end, count)?;
    if position != content.len() {
        return Err(RecoverySourceLeaseDenial::LeaseConflict);
    }
    Ok(PersistedRecoverySourceLease {
        kind,
        operation_identity,
        source_identity,
        source_evidence_identity,
        source_root,
        artifact_names,
    })
}

fn decode_names(
    content: &[u8],
    mut position: usize,
    count: usize,
) -> Result<(Vec<String>, usize), RecoverySourceLeaseDenial> {
    let mut names = Vec::new();
    names
        .try_reserve_exact(count)
        .map_err(|_| RecoverySourceLeaseDenial::AllocationFailed)?;
    for _ in 0..count {
        let length_end = position
            .checked_add(4)
            .ok_or(RecoverySourceLeaseDenial::LeaseConflict)?;
        let length_bytes = content
            .get(position..length_end)
            .ok_or(RecoverySourceLeaseDenial::LeaseConflict)?;
        let length = usize::try_from(u32::from_le_bytes(
            length_bytes.try_into().expect("fixed name length"),
        ))
        .map_err(|_| RecoverySourceLeaseDenial::LeaseConflict)?;
        let end = length_end
            .checked_add(length)
            .ok_or(RecoverySourceLeaseDenial::LeaseConflict)?;
        let bytes = content
            .get(length_end..end)
            .ok_or(RecoverySourceLeaseDenial::LeaseConflict)?;
        names.push(
            String::from_utf8(bytes.to_vec())
                .map_err(|_| RecoverySourceLeaseDenial::LeaseConflict)?,
        );
        position = end;
    }
    Ok((names, position))
}

pub(super) fn filename(identity: [u8; 32]) -> String {
    let mut name = String::from("source-lease-");
    for byte in identity {
        name.push_str(&format!("{byte:02x}"));
    }
    name.push_str(".record");
    name
}

pub(super) fn identity_from_filename(name: &str) -> Option<[u8; 32]> {
    let hex = name
        .strip_prefix("source-lease-")?
        .strip_suffix(".record")?;
    if hex.len() != 64 {
        return None;
    }
    let mut identity = [0; 32];
    for (index, byte) in identity.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(identity)
}

pub(super) fn persist(
    directory: &Path,
    final_path: &Path,
    content: &[u8],
) -> Result<(), RecoverySourceLeaseDenial> {
    if final_path.exists() {
        return verify_existing(final_path, content);
    }
    let pending_path = directory.join(format!(
        ".pending-source-lease-{}-{}",
        std::process::id(),
        NEXT_PENDING_RECORD.fetch_add(1, Ordering::Relaxed),
    ));
    let mut pending = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&pending_path)?;
    pending.write_all(content)?;
    pending.sync_all()?;
    match std::fs::hard_link(&pending_path, final_path) {
        Ok(()) => {
            sync_directory(directory)?;
            std::fs::remove_file(&pending_path)?;
            sync_directory(directory)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            std::fs::remove_file(&pending_path)?;
            verify_existing(final_path, content)
        }
        Err(error) => Err(RecoverySourceLeaseDenial::Io(error)),
    }
}

fn verify_existing(path: &Path, expected: &[u8]) -> Result<(), RecoverySourceLeaseDenial> {
    if std::fs::read(path)? == expected {
        Ok(())
    } else {
        Err(RecoverySourceLeaseDenial::LeaseConflict)
    }
}

#[cfg(windows)]
fn encode_path(path: &Path) -> (u8, Vec<u8>) {
    use std::os::windows::ffi::OsStrExt;
    (
        1,
        path.as_os_str()
            .encode_wide()
            .flat_map(u16::to_le_bytes)
            .collect(),
    )
}

#[cfg(windows)]
fn decode_path(platform: u8, bytes: &[u8]) -> Result<PathBuf, RecoverySourceLeaseDenial> {
    use std::os::windows::ffi::OsStringExt;
    if platform != 1 || !bytes.len().is_multiple_of(2) {
        return Err(RecoverySourceLeaseDenial::LeaseConflict);
    }
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    Ok(std::ffi::OsString::from_wide(&units).into())
}

#[cfg(not(windows))]
fn encode_path(path: &Path) -> (u8, Vec<u8>) {
    use std::os::unix::ffi::OsStrExt;
    (2, path.as_os_str().as_bytes().to_vec())
}

#[cfg(not(windows))]
fn decode_path(platform: u8, bytes: &[u8]) -> Result<PathBuf, RecoverySourceLeaseDenial> {
    use std::os::unix::ffi::OsStringExt;
    if platform != 2 {
        return Err(RecoverySourceLeaseDenial::LeaseConflict);
    }
    Ok(std::ffi::OsString::from_vec(bytes.to_vec()).into())
}
