use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::CanonicalPlatformPulse;

static INSTALLATION_ORDINAL: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PulseInstallationPath {
    root: PathBuf,
}

#[derive(Debug)]
pub(crate) struct IsolatedPulseInstallation {
    root: PathBuf,
    cleanup_required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PulseInstallationCleanupEvidence {
    removed_owned_root: bool,
}

#[derive(Debug)]
pub(crate) enum PulseInstallationFailure {
    CreateRoot(std::io::Error),
    PrepareEntrySource {
        primary: PulseEntrySourcePreparationFailure,
        rollback: Result<PulseInstallationCleanupEvidence, PulseInstallationCleanupFailure>,
    },
    PrepareIntentSource {
        primary: PulseEntrySourcePreparationFailure,
        rollback: Result<PulseInstallationCleanupEvidence, PulseInstallationCleanupFailure>,
    },
    PreparePortalCancelSource {
        primary: PulseEntrySourcePreparationFailure,
        rollback: Result<PulseInstallationCleanupEvidence, PulseInstallationCleanupFailure>,
    },
    PreparePortalPrimarySource {
        primary: PulseEntrySourcePreparationFailure,
        rollback: Result<PulseInstallationCleanupEvidence, PulseInstallationCleanupFailure>,
    },
}

#[derive(Debug)]
pub(crate) enum PulseEntrySourcePreparationFailure {
    Create(std::io::Error),
    Write(std::io::Error),
    Flush(std::io::Error),
}

#[derive(Debug)]
pub(crate) enum PulseInstallationCleanupFailure {
    RemoveRoot(std::io::Error),
    Residue(PathBuf),
}

impl fmt::Display for PulseInstallationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateRoot(error) => write!(formatter, "create isolated root: {error}"),
            Self::PrepareEntrySource { primary, rollback } => {
                write!(
                    formatter,
                    "prepare isolated main.wui: {primary}; rollback: "
                )?;
                match rollback {
                    Ok(evidence) => {
                        write!(formatter, "released={}", evidence.removed_owned_root)
                    }
                    Err(failure) => write!(formatter, "failed({failure})"),
                }
            }
            Self::PrepareIntentSource { primary, rollback } => {
                write!(
                    formatter,
                    "prepare isolated intent source: {primary}; rollback: "
                )?;
                match rollback {
                    Ok(evidence) => write!(formatter, "released={}", evidence.removed_owned_root),
                    Err(failure) => write!(formatter, "failed({failure})"),
                }
            }
            Self::PreparePortalCancelSource { primary, rollback } => {
                write!(
                    formatter,
                    "prepare isolated portal_cancel.wui: {primary}; rollback: "
                )?;
                match rollback {
                    Ok(evidence) => write!(formatter, "released={}", evidence.removed_owned_root),
                    Err(failure) => write!(formatter, "failed({failure})"),
                }
            }
            Self::PreparePortalPrimarySource { primary, rollback } => {
                write!(
                    formatter,
                    "prepare isolated portal_action.wui: {primary}; rollback: "
                )?;
                match rollback {
                    Ok(evidence) => write!(formatter, "released={}", evidence.removed_owned_root),
                    Err(failure) => write!(formatter, "failed({failure})"),
                }
            }
        }
    }
}

impl fmt::Display for PulseEntrySourcePreparationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create(error) => {
                write!(formatter, "create isolated main.wui: {error}")
            }
            Self::Write(error) => {
                write!(formatter, "write isolated main.wui: {error}")
            }
            Self::Flush(error) => {
                write!(formatter, "flush isolated main.wui: {error}")
            }
        }
    }
}

impl fmt::Display for PulseInstallationCleanupFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RemoveRoot(error) => write!(formatter, "remove isolated root: {error}"),
            Self::Residue(path) => {
                write!(
                    formatter,
                    "isolated root remained after cleanup: {}",
                    path.display()
                )
            }
        }
    }
}

impl IsolatedPulseInstallation {
    pub(crate) fn install(
        canonical: CanonicalPlatformPulse,
    ) -> Result<Self, PulseInstallationFailure> {
        let path = PulseInstallationPath::fresh();
        Self::install_at(canonical, &path)
    }

    pub(crate) fn install_at(
        canonical: CanonicalPlatformPulse,
        path: &PulseInstallationPath,
    ) -> Result<Self, PulseInstallationFailure> {
        let root = path.root.clone();
        fs::create_dir(&root).map_err(PulseInstallationFailure::CreateRoot)?;
        let mut installation = Self {
            root,
            cleanup_required: true,
        };
        if let Err(primary) = installation.write_source("main.wui", canonical.source_bytes()) {
            let rollback = installation.close();
            return Err(PulseInstallationFailure::PrepareEntrySource { primary, rollback });
        }
        if let Err(primary) =
            installation.write_source("portal_action.wui", canonical.portal_primary_source_bytes())
        {
            let rollback = installation.close();
            return Err(PulseInstallationFailure::PreparePortalPrimarySource { primary, rollback });
        }
        if let Err(primary) =
            installation.write_source("portal_cancel.wui", canonical.portal_cancel_source_bytes())
        {
            let rollback = installation.close();
            return Err(PulseInstallationFailure::PreparePortalCancelSource { primary, rollback });
        }
        if let Err(primary) = installation.write_source(
            "platform-pulse-intent.json",
            canonical.intent_source_bytes(),
        ) {
            let rollback = installation.close();
            return Err(PulseInstallationFailure::PrepareIntentSource { primary, rollback });
        }
        if let Err(primary) =
            installation.write_source("modal_review.wui", canonical.modal_review_source_bytes())
        {
            let rollback = installation.close();
            return Err(PulseInstallationFailure::PrepareEntrySource { primary, rollback });
        }
        Ok(installation)
    }

    pub(crate) fn source_root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn entry_source(&self) -> PathBuf {
        self.root.join("main.wui")
    }

    pub(crate) fn intent_source(&self) -> PathBuf {
        self.root.join("platform-pulse-intent.json")
    }

    pub(crate) fn portal_cancel_source(&self) -> PathBuf {
        self.root.join("portal_cancel.wui")
    }

    pub(crate) fn portal_primary_source(&self) -> PathBuf {
        self.root.join("portal_action.wui")
    }

    pub(crate) fn failure_source_snapshot(&self) -> Option<Box<[u8]>> {
        fs::read(self.entry_source())
            .ok()
            .map(Vec::into_boxed_slice)
    }

    pub(crate) fn close(
        &mut self,
    ) -> Result<PulseInstallationCleanupEvidence, PulseInstallationCleanupFailure> {
        fs::remove_dir_all(&self.root).map_err(PulseInstallationCleanupFailure::RemoveRoot)?;
        self.cleanup_required = false;
        if self.root.exists() {
            return Err(PulseInstallationCleanupFailure::Residue(self.root.clone()));
        }
        Ok(PulseInstallationCleanupEvidence {
            removed_owned_root: true,
        })
    }

    fn write_source(
        &self,
        name: &str,
        source: &[u8],
    ) -> Result<(), PulseEntrySourcePreparationFailure> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(self.root.join(name))
            .map_err(PulseEntrySourcePreparationFailure::Create)?;
        file.write_all(source)
            .map_err(PulseEntrySourcePreparationFailure::Write)?;
        file.flush()
            .map_err(PulseEntrySourcePreparationFailure::Flush)
    }
}

impl PulseInstallationPath {
    pub(crate) fn fresh() -> Self {
        let ordinal = INSTALLATION_ORDINAL.fetch_add(1, Ordering::Relaxed);
        Self {
            root: std::env::temp_dir().join(format!(
                "worth-ui-platform-pulse-executable-world-{}-{ordinal}",
                std::process::id()
            )),
        }
    }

    pub(crate) fn is_absent(&self) -> bool {
        !self.root.exists()
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
}

impl PulseInstallationCleanupEvidence {
    pub(crate) fn removed_owned_root(self) -> bool {
        self.removed_owned_root
    }
}

impl Drop for IsolatedPulseInstallation {
    fn drop(&mut self) {
        if self.cleanup_required {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IsolatedPulseInstallation, PulseInstallationPath};
    use crate::installation::CanonicalPlatformPulse;

    #[test]
    fn isolated_installation_owns_exact_source_and_explicit_cleanup() {
        let mut installation =
            IsolatedPulseInstallation::install(CanonicalPlatformPulse::checked_in())
                .expect("installation");
        let source = std::fs::read(installation.source_root().join("main.wui")).expect("source");
        assert_eq!(source, CanonicalPlatformPulse::checked_in().source_bytes());
        assert_eq!(
            std::fs::read(installation.portal_primary_source()).expect("portal primary source"),
            CanonicalPlatformPulse::checked_in().portal_primary_source_bytes()
        );
        assert_eq!(
            std::fs::read(installation.portal_cancel_source()).expect("portal cancel source"),
            CanonicalPlatformPulse::checked_in().portal_cancel_source_bytes()
        );
        assert!(installation.close().expect("cleanup").removed_owned_root());
    }

    #[test]
    fn one_path_accepts_two_fresh_installations_after_proven_cleanup() {
        let path = PulseInstallationPath::fresh();
        let mut predecessor =
            IsolatedPulseInstallation::install_at(CanonicalPlatformPulse::checked_in(), &path)
                .expect("predecessor installation");
        assert!(predecessor
            .close()
            .expect("predecessor cleanup")
            .removed_owned_root());
        assert!(path.is_absent());

        let mut successor =
            IsolatedPulseInstallation::install_at(CanonicalPlatformPulse::checked_in(), &path)
                .expect("successor installation");
        assert!(successor
            .close()
            .expect("successor cleanup")
            .removed_owned_root());
        assert!(path.is_absent());
    }
}
