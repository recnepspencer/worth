use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use fs4::FileExt;
use sha2::{Digest, Sha256};

use super::PhysicalBackupMaterializationDenial;
use super::{io_denial, reject_symbolic_link, reject_symbolic_link_if_present};
use crate::PhysicalBackupSource;

pub(super) const SESSION_DESCRIPTOR_NAME: &str = "materialization.session";
const LOCK_DIRECTORY_NAME: &str = ".worth-store-backup-session-locks";
const DESCRIPTOR_MAGIC: &[u8; 8] = b"WSTRBKSN";
const DESCRIPTOR_VERSION: u16 = 1;
const DESCRIPTOR_BYTES: usize = 8 + 2 + 8 + 32;

pub(super) struct PhysicalBackupSessionIdentityGuard {
    _lock: File,
}

impl PhysicalBackupSessionIdentityGuard {
    pub(super) fn acquire(
        target_parent: &Path,
        session_identity: &str,
    ) -> Result<Self, PhysicalBackupMaterializationDenial> {
        std::fs::create_dir_all(target_parent)
            .map_err(|source| io_denial(target_parent, source))?;
        reject_symbolic_link(target_parent)?;
        let lock_directory = target_parent.join(LOCK_DIRECTORY_NAME);
        std::fs::create_dir_all(&lock_directory)
            .map_err(|source| io_denial(&lock_directory, source))?;
        reject_symbolic_link(&lock_directory)?;
        let lock_path = lock_directory.join(format!("{session_identity}.lock"));
        reject_symbolic_link_if_present(&lock_path)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|source| io_denial(&lock_path, source))?;
        lock.try_lock_exclusive().map_err(|source| {
            let contended = fs4::lock_contended_error();
            if source.kind() == std::io::ErrorKind::WouldBlock
                || source.raw_os_error() == contended.raw_os_error()
            {
                PhysicalBackupMaterializationDenial::SessionBusy {
                    session_identity: session_identity.to_owned(),
                }
            } else {
                io_denial(&lock_path, source)
            }
        })?;

        Ok(Self { _lock: lock })
    }

    pub(super) fn bind_source_set(
        &self,
        staging_root: &Path,
        session_identity: &str,
        sources: &[PhysicalBackupSource],
    ) -> Result<u64, PhysicalBackupMaterializationDenial> {
        admit_or_create_descriptor(staging_root, session_identity, sources)
    }
}

impl Drop for PhysicalBackupSessionIdentityGuard {
    fn drop(&mut self) {
        // A concurrent fork can inherit the open-file description until exec. Release
        // this owner's lock now; closing the file alone waits for every duplicate.
        // Drop has no error channel, and file close remains the fallback on failure.
        let _ = FileExt::unlock(&self._lock);
    }
}

fn admit_or_create_descriptor(
    staging_root: &Path,
    session_identity: &str,
    sources: &[PhysicalBackupSource],
) -> Result<u64, PhysicalBackupMaterializationDenial> {
    let descriptor = encode_descriptor(session_identity, sources)?;
    let path = staging_root.join(SESSION_DESCRIPTOR_NAME);
    reject_symbolic_link_if_present(&path)?;
    let pending_manifest = staging_root.join("backup.manifest.pending");
    let published_manifest = staging_root.join("backup.manifest");
    if !path.exists() {
        match (pending_manifest.exists(), published_manifest.exists()) {
            (false, true) => return Ok(0),
            (true, false) => {
                return Err(PhysicalBackupMaterializationDenial::SessionIdentityMismatch { path });
            }
            (true, true) => {
                return Err(
                    PhysicalBackupMaterializationDenial::ExistingPublicationMismatch {
                        path: pending_manifest,
                    },
                );
            }
            (false, false) => {}
        }
    }
    match OpenOptions::new().create_new(true).write(true).open(&path) {
        Ok(mut file) => {
            file.write_all(&descriptor)
                .map_err(|source| io_denial(&path, source))?;
            file.sync_all().map_err(|source| io_denial(&path, source))?;
            crate::directory_durability::sync_directory(staging_root)
                .map_err(|source| io_denial(staging_root, source))?;
            Ok(2)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            require_exact_descriptor(&path, &descriptor).map(|()| 0)
        }
        Err(source) => Err(io_denial(&path, source)),
    }
}

fn require_exact_descriptor(
    path: &Path,
    expected: &[u8; DESCRIPTOR_BYTES],
) -> Result<(), PhysicalBackupMaterializationDenial> {
    let mut file = File::open(path).map_err(|source| io_denial(path, source))?;
    let mut actual = [0_u8; DESCRIPTOR_BYTES];
    file.read_exact(&mut actual).map_err(|_| {
        PhysicalBackupMaterializationDenial::SessionIdentityMismatch {
            path: path.to_path_buf(),
        }
    })?;
    let mut trailing = [0_u8; 1];
    if actual != *expected
        || file
            .read(&mut trailing)
            .map_err(|source| io_denial(path, source))?
            != 0
    {
        return Err(
            PhysicalBackupMaterializationDenial::SessionIdentityMismatch {
                path: path.to_path_buf(),
            },
        );
    }
    Ok(())
}

fn encode_descriptor(
    session_identity: &str,
    sources: &[PhysicalBackupSource],
) -> Result<[u8; DESCRIPTOR_BYTES], PhysicalBackupMaterializationDenial> {
    let source_count = u64::try_from(sources.len())
        .map_err(|_| PhysicalBackupMaterializationDenial::CounterOverflow)?;
    let mut fingerprint = Sha256::new();
    fingerprint.update(b"worth-store/physical-backup-session/v1");
    update_field(&mut fingerprint, session_identity.as_bytes())?;
    fingerprint.update(source_count.to_le_bytes());
    for source in sources {
        update_field(&mut fingerprint, source.output_name().as_bytes())?;
        fingerprint.update(source.expected_bytes().to_le_bytes());
        fingerprint.update(source.expected_digest());
        fingerprint.update(source.expected_physical_identity());
    }
    let mut encoded = [0_u8; DESCRIPTOR_BYTES];
    encoded[..8].copy_from_slice(DESCRIPTOR_MAGIC);
    encoded[8..10].copy_from_slice(&DESCRIPTOR_VERSION.to_le_bytes());
    encoded[10..18].copy_from_slice(&source_count.to_le_bytes());
    encoded[18..].copy_from_slice(&fingerprint.finalize());
    Ok(encoded)
}

fn update_field(
    fingerprint: &mut Sha256,
    bytes: &[u8],
) -> Result<(), PhysicalBackupMaterializationDenial> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| PhysicalBackupMaterializationDenial::CounterOverflow)?;
    fingerprint.update(length.to_le_bytes());
    fingerprint.update(bytes);
    Ok(())
}

pub(super) fn descriptor_path(staging_root: &Path) -> PathBuf {
    staging_root.join(SESSION_DESCRIPTOR_NAME)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn dropping_session_releases_ownership_even_with_a_duplicated_descriptor() {
        let directory = tempfile::tempdir().expect("directory");
        let guard = PhysicalBackupSessionIdentityGuard::acquire(directory.path(), "session")
            .expect("session owner");
        // A concurrently forked process temporarily inherits this same open-file description.
        let inherited = guard._lock.try_clone().expect("duplicate descriptor");
        drop(guard);
        let next = PhysicalBackupSessionIdentityGuard::acquire(directory.path(), "session")
            .expect("the live owner released its lock before the duplicate closes");
        drop(inherited);
        drop(next);
    }
}
