use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use super::{ControlMediaFault, ControlMediaLocation};

const OBJECT_READ_BUFFER_BYTES: usize = 64 * 1024;
static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ControlRecoveryObjectHandle {
    digest: [u8; 32],
    bytes: u64,
}

impl ControlRecoveryObjectHandle {
    pub fn for_content(content: &[u8]) -> Self {
        Self {
            digest: Sha256::digest(content).into(),
            bytes: content.len() as u64,
        }
    }

    pub const fn from_persisted(digest: [u8; 32], bytes: u64) -> Option<Self> {
        if bytes == 0 {
            None
        } else {
            Some(Self { digest, bytes })
        }
    }

    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub const fn bytes(self) -> u64 {
        self.bytes
    }
}

#[derive(Debug)]
pub(crate) struct PhysicalControlRecoveryObjectStore {
    root: PathBuf,
}

impl PhysicalControlRecoveryObjectStore {
    pub(crate) fn open(location: &ControlMediaLocation) -> Result<Self, ControlMediaFault> {
        let root = location.recovery_object_root();
        let existed = root.exists();
        std::fs::create_dir_all(&root)?;
        if !existed && let Some(parent) = root.parent() {
            crate::directory_durability::sync_directory(parent)?;
        }
        Ok(Self { root })
    }

    pub(crate) fn publish(
        &self,
        content: &[u8],
    ) -> Result<ControlRecoveryObjectHandle, ControlMediaFault> {
        if content.is_empty() {
            return Err(ControlMediaFault::EmptyRecoveryObject);
        }
        let handle = ControlRecoveryObjectHandle::for_content(content);
        let final_path = self.path(handle);
        if final_path.exists() {
            self.verify(handle)?;
            return Ok(handle);
        }

        let staging = self.unique_staging_path(handle);
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&staging)?;
        if let Err(error) = write_and_sync(&mut file, content) {
            drop(file);
            let _ = std::fs::remove_file(&staging);
            return Err(error.into());
        }
        drop(file);

        match std::fs::rename(&staging, &final_path) {
            Ok(()) => crate::directory_durability::sync_directory(&self.root)?,
            Err(_error) if final_path.exists() => {
                let _ = std::fs::remove_file(&staging);
                self.verify(handle)?;
                return Ok(handle);
            }
            Err(error) => {
                let _ = std::fs::remove_file(&staging);
                return Err(error.into());
            }
        }
        self.verify(handle)?;
        Ok(handle)
    }

    pub(crate) fn read(
        &self,
        handle: ControlRecoveryObjectHandle,
    ) -> Result<Vec<u8>, ControlMediaFault> {
        if handle.bytes() == 0 {
            return Err(ControlMediaFault::EmptyRecoveryObject);
        }
        let capacity = usize::try_from(handle.bytes()).map_err(|_| {
            ControlMediaFault::RecoveryObjectLengthMismatch {
                digest: handle.digest(),
                expected: handle.bytes(),
                actual: u64::MAX,
            }
        })?;
        let mut file = match File::open(self.path(handle)) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(ControlMediaFault::MissingRecoveryObject {
                    digest: handle.digest(),
                })
            }
            Err(error) => return Err(error.into()),
        };
        let actual = file.metadata()?.len();
        if actual != handle.bytes() {
            return Err(ControlMediaFault::RecoveryObjectLengthMismatch {
                digest: handle.digest(),
                expected: handle.bytes(),
                actual,
            });
        }
        let mut content = Vec::new();
        content
            .try_reserve_exact(capacity)
            .map_err(|_| ControlMediaFault::AllocationFailed)?;
        content.resize(capacity, 0);
        file.read_exact(&mut content)?;
        let mut extra = [0; 1];
        if file.read(&mut extra)? != 0 {
            return Err(ControlMediaFault::RecoveryObjectLengthMismatch {
                digest: handle.digest(),
                expected: handle.bytes(),
                actual: handle.bytes().saturating_add(1),
            });
        }
        if <[u8; 32]>::from(Sha256::digest(&content)) != handle.digest() {
            return Err(ControlMediaFault::CorruptRecoveryObject {
                digest: handle.digest(),
            });
        }
        Ok(content)
    }

    fn verify(&self, handle: ControlRecoveryObjectHandle) -> Result<(), ControlMediaFault> {
        let path = self.path(handle);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(ControlMediaFault::MissingRecoveryObject {
                    digest: handle.digest(),
                })
            }
            Err(error) => return Err(error.into()),
        };
        let actual = file.metadata()?.len();
        if actual != handle.bytes() {
            return Err(ControlMediaFault::RecoveryObjectLengthMismatch {
                digest: handle.digest(),
                expected: handle.bytes(),
                actual,
            });
        }
        let mut digest = Sha256::new();
        let mut bytes = [0; OBJECT_READ_BUFFER_BYTES];
        loop {
            let read = file.read(&mut bytes)?;
            if read == 0 {
                break;
            }
            digest.update(&bytes[..read]);
        }
        if <[u8; 32]>::from(digest.finalize()) != handle.digest() {
            return Err(ControlMediaFault::CorruptRecoveryObject {
                digest: handle.digest(),
            });
        }
        Ok(())
    }

    fn path(&self, handle: ControlRecoveryObjectHandle) -> PathBuf {
        self.root.join(hex(&handle.digest()))
    }

    fn unique_staging_path(&self, handle: ControlRecoveryObjectHandle) -> PathBuf {
        let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        self.root.join(format!(
            ".{}.{}.{}.staging",
            hex(&handle.digest()),
            std::process::id(),
            sequence
        ))
    }
}

fn write_and_sync(file: &mut File, content: &[u8]) -> std::io::Result<()> {
    file.write_all(content)?;
    file.sync_all()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
