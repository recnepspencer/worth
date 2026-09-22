use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::installation::IsolatedPulseInstallation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PulseSourceDeltaIdentity {
    QueryStatusV1,
    QueryStatusV2,
    Green,
    Malformed,
    CanonicalBlueRecovery,
    RevisionSchema,
    StatusSchemaRecovery,
    IntentReadyReleased,
    IntentQueryDenialRequested,
    PortalFocusFallback,
    IntentConfirmationHeld,
    IntentConfirmationReleased,
    IntentDisabled,
    IntentDenied,
    IntentFinalHeld,
    IntentRouteRemoved,
}

#[derive(Debug)]
pub(crate) struct AppliedPulseSourceDelta<Kind> {
    identity: PulseSourceDeltaIdentity,
    written_bytes: usize,
    content_fingerprint: u64,
    entry_source: PathBuf,
    _kind: PhantomData<Kind>,
}

#[derive(Debug)]
pub(crate) enum PulseSourceActionFailure {
    CreateTemporary(std::io::Error),
    WriteTemporary(std::io::Error),
    FlushTemporary(std::io::Error),
    SyncTemporary(std::io::Error),
    /// `MoveFileEx` takes UTF-16; a path that is not Unicode cannot be named
    /// to it. Other targets replace by `rename`, which takes any byte path.
    #[cfg(target_os = "windows")]
    NonUnicodeWindowsPath(PathBuf),
    AtomicReplace {
        primary: String,
        temporary_cleanup: Result<(), std::io::Error>,
    },
    ReadBack(std::io::Error),
    ReadBackMismatch,
}

impl fmt::Display for PulseSourceActionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateTemporary(error) => write!(formatter, "create delta file: {error}"),
            Self::WriteTemporary(error) => write!(formatter, "write delta file: {error}"),
            Self::FlushTemporary(error) => write!(formatter, "flush delta file: {error}"),
            Self::SyncTemporary(error) => write!(formatter, "sync delta file: {error}"),
            #[cfg(target_os = "windows")]
            Self::NonUnicodeWindowsPath(path) => {
                write!(
                    formatter,
                    "atomic Windows path is not Unicode: {}",
                    path.display()
                )
            }
            Self::AtomicReplace {
                primary,
                temporary_cleanup,
            } => write!(
                formatter,
                "atomically replace source: {primary}; temporary cleanup: {temporary_cleanup:?}"
            ),
            Self::ReadBack(error) => write!(formatter, "read replaced source: {error}"),
            Self::ReadBackMismatch => {
                formatter.write_str("replaced source bytes differ from the named delta")
            }
        }
    }
}

pub(super) fn apply<Kind>(
    installation: &IsolatedPulseInstallation,
    identity: PulseSourceDeltaIdentity,
    bytes: &[u8],
) -> Result<AppliedPulseSourceDelta<Kind>, PulseSourceActionFailure> {
    apply_path(installation.entry_source(), identity, bytes)
}

pub(super) fn apply_path<Kind>(
    destination: PathBuf,
    identity: PulseSourceDeltaIdentity,
    bytes: &[u8],
) -> Result<AppliedPulseSourceDelta<Kind>, PulseSourceActionFailure> {
    let temporary = destination.with_extension("replacement");
    write_temporary(&temporary, bytes)?;
    replace_source(&destination, &temporary)?;
    let observed = fs::read(&destination).map_err(PulseSourceActionFailure::ReadBack)?;
    if observed != bytes {
        return Err(PulseSourceActionFailure::ReadBackMismatch);
    }
    Ok(AppliedPulseSourceDelta {
        identity,
        written_bytes: bytes.len(),
        content_fingerprint: fingerprint(bytes),
        entry_source: destination,
        _kind: PhantomData,
    })
}

fn write_temporary(path: &Path, bytes: &[u8]) -> Result<(), PulseSourceActionFailure> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(PulseSourceActionFailure::CreateTemporary)?;
    file.write_all(bytes)
        .map_err(PulseSourceActionFailure::WriteTemporary)?;
    file.flush()
        .map_err(PulseSourceActionFailure::FlushTemporary)?;
    file.sync_all()
        .map_err(PulseSourceActionFailure::SyncTemporary)
}

#[cfg(target_os = "windows")]
fn replace_source(destination: &Path, replacement: &Path) -> Result<(), PulseSourceActionFailure> {
    if !destination.exists() {
        return fs::rename(replacement, destination).map_err(|error| {
            PulseSourceActionFailure::AtomicReplace {
                primary: error.to_string(),
                temporary_cleanup: fs::remove_file(replacement),
            }
        });
    }
    let destination_text = destination
        .to_str()
        .ok_or_else(|| PulseSourceActionFailure::NonUnicodeWindowsPath(destination.to_owned()))?;
    let replacement_text = replacement
        .to_str()
        .ok_or_else(|| PulseSourceActionFailure::NonUnicodeWindowsPath(replacement.to_owned()))?;
    winsafe::ReplaceFile(
        destination_text,
        replacement_text,
        None,
        winsafe::co::REPLACEFILE::WRITE_THROUGH,
    )
    .map_err(|error| PulseSourceActionFailure::AtomicReplace {
        primary: error.to_string(),
        temporary_cleanup: fs::remove_file(replacement),
    })
}

#[cfg(not(target_os = "windows"))]
fn replace_source(destination: &Path, replacement: &Path) -> Result<(), PulseSourceActionFailure> {
    fs::rename(replacement, destination).map_err(|error| PulseSourceActionFailure::AtomicReplace {
        primary: error.to_string(),
        temporary_cleanup: fs::remove_file(replacement),
    })
}

fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |digest, byte| {
        (digest ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

impl<Kind> AppliedPulseSourceDelta<Kind> {
    pub(crate) fn identity(&self) -> PulseSourceDeltaIdentity {
        self.identity
    }

    pub(crate) fn action_count(&self) -> u32 {
        1
    }

    pub(crate) fn written_bytes(&self) -> usize {
        self.written_bytes
    }

    pub(crate) fn content_fingerprint(&self) -> u64 {
        self.content_fingerprint
    }

    pub(crate) fn entry_source(&self) -> &Path {
        &self.entry_source
    }
}
