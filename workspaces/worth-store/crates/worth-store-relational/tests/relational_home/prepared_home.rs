//! A cleanly closed home holding a script prefix, built fresh for every case
//! that starts from it. A Store binds its files' identities, so a copied
//! directory is refused by recovery as damaged (`BindingFreshness`); each
//! case therefore runs the prefix itself.

use std::path::{Path, PathBuf};

use worth_execution::ExecutionRequest;

use super::durable_run::{apply, open};
use super::workload::Step;

/// A closed home after a prefix; removed when dropped.
pub(crate) struct PreparedHome {
    _directory: tempfile::TempDir,
    location: PathBuf,
}

impl PreparedHome {
    /// A new home after `steps`, closed.
    pub(crate) fn prepare(steps: &[Step], execution: &ExecutionRequest<'_, '_>) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let location = directory.path().join("home");
        let home = open(&location, execution).expect("a new home opens");
        apply(&home, execution, steps);
        home.close();
        Self {
            _directory: directory,
            location,
        }
    }

    pub(crate) fn location(&self) -> &Path {
        &self.location
    }
}
