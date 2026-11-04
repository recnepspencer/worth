use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use worth_foundational::CanonicalDigestId;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::wire_encoding;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessArtifactObservation {
    Absent,
    File {
        content_sha256: CanonicalDigestId,
    },
    Directory {
        tree_sha256: CanonicalDigestId,
        entry_count: usize,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessArtifactDisposition {
    InputSnapshot,
    OutputChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProcessArtifactPath {
    purpose: String,
    path: String,
    disposition: ProcessArtifactDisposition,
    initial_observation: ProcessArtifactObservation,
}

impl ProcessArtifactPath {
    pub fn new(purpose: impl Into<String>, path: impl AsRef<Path>) -> Result<Self, String> {
        Self::construct(purpose, path, ProcessArtifactDisposition::InputSnapshot)
    }

    pub fn output_channel(
        purpose: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, String> {
        Self::construct(purpose, path, ProcessArtifactDisposition::OutputChannel)
    }

    fn construct(
        purpose: impl Into<String>,
        path: impl AsRef<Path>,
        disposition: ProcessArtifactDisposition,
    ) -> Result<Self, String> {
        let purpose = purpose.into();
        if purpose.trim().is_empty() {
            return Err("process artifact purpose cannot be empty".to_owned());
        }
        let path = absolute_path(path.as_ref())?;
        let initial_observation = observe(&path)?;
        if disposition == ProcessArtifactDisposition::OutputChannel
            && initial_observation != ProcessArtifactObservation::Absent
        {
            return Err(format!(
                "process output channel {purpose} already exists at {}",
                path.display()
            ));
        }
        Ok(Self {
            purpose,
            path: normalized_path(&path),
            disposition,
            initial_observation,
        })
    }

    pub fn purpose(&self) -> &str {
        &self.purpose
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub const fn initial_observation(&self) -> &ProcessArtifactObservation {
        &self.initial_observation
    }

    pub const fn disposition(&self) -> ProcessArtifactDisposition {
        self.disposition
    }

    pub(crate) fn validate_child_admission(&self) -> Result<(), String> {
        match self.disposition {
            ProcessArtifactDisposition::InputSnapshot => {
                let observed = Self::new(&self.purpose, &self.path)?;
                if observed == *self {
                    Ok(())
                } else {
                    Err(format!(
                        "process input artifact {} changed before child admission",
                        self.purpose
                    ))
                }
            }
            ProcessArtifactDisposition::OutputChannel => {
                if self.initial_observation == ProcessArtifactObservation::Absent
                    && observe(Path::new(&self.path))? == ProcessArtifactObservation::Absent
                {
                    Ok(())
                } else {
                    Err(format!(
                        "process output channel {} was not initially absent",
                        self.purpose
                    ))
                }
            }
        }
    }

    pub(crate) fn admits_output_path(&self, path: &Path) -> Result<bool, String> {
        Ok(
            self.disposition == ProcessArtifactDisposition::OutputChannel
                && self.path == normalized_path(&absolute_path(path)?),
        )
    }
}

pub(crate) fn output_artifact_identity(path: &Path) -> Result<[u8; 32], String> {
    let observation = observe(&absolute_path(path)?)?;
    if observation == ProcessArtifactObservation::Absent {
        return Err(format!(
            "process output artifact {} is absent",
            path.display()
        ));
    }
    wire_encoding::encode(&("worth-store-process-output-artifact-v1", observation))
        .map(|bytes| Sha256::digest(bytes).into())
}

fn observe(path: &Path) -> Result<ProcessArtifactObservation, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ProcessArtifactObservation::Absent);
        }
        Err(error) => return Err(format!("could not inspect {}: {error}", path.display())),
    };
    if metadata.is_file() {
        return Ok(ProcessArtifactObservation::File {
            content_sha256: CanonicalDigestId::new(file_digest(path)?),
        });
    }
    if !metadata.is_dir() {
        return Err(format!(
            "process artifact {} is neither a regular file nor a directory",
            path.display()
        ));
    }
    let mut entries = Vec::new();
    collect_directory(path, path, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut digest = Sha256::new();
    digest.update(b"worth-store-process-artifact-directory-v1");
    for (relative, kind, identity) in &entries {
        digest.update((relative.len() as u64).to_be_bytes());
        digest.update(relative.as_bytes());
        digest.update([*kind]);
        digest.update(identity);
    }
    Ok(ProcessArtifactObservation::Directory {
        tree_sha256: CanonicalDigestId::new(digest.finalize().into()),
        entry_count: entries.len(),
    })
}

fn collect_directory(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<(String, u8, [u8; 32])>,
) -> Result<(), String> {
    let mut children = fs::read_dir(directory)
        .map_err(|error| format!("could not inspect {}: {error}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not inspect {}: {error}", directory.display()))?;
    children.sort();
    for child in children {
        let metadata = fs::symlink_metadata(&child)
            .map_err(|error| format!("could not inspect {}: {error}", child.display()))?;
        let relative = child
            .strip_prefix(root)
            .map_err(|_| format!("{} escaped artifact root", child.display()))?;
        let relative = normalized_path(relative);
        if metadata.file_type().is_symlink() {
            let target = fs::read_link(&child)
                .map_err(|error| format!("could not inspect {}: {error}", child.display()))?;
            entries.push((relative, 3, Sha256::digest(normalized_path(&target)).into()));
        } else if metadata.is_dir() {
            entries.push((relative, 2, Sha256::digest(b"directory").into()));
            collect_directory(root, &child, entries)?;
        } else if metadata.is_file() {
            entries.push((relative, 1, file_digest(&child)?));
        } else {
            return Err(format!(
                "process artifact {} contains an unsupported entry",
                child.display()
            ));
        }
    }
    Ok(())
}

fn file_digest(path: &Path) -> Result<[u8; 32], String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest.finalize().into())
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|root| root.join(path))
            .map_err(|error| format!("process artifact working directory is unavailable: {error}"))
    }
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
