//! Temporary governed repository writer for authority sealing proofs.

use std::fs;
use std::path::PathBuf;

mod assembly;
mod configuration;
mod entry_crate;
mod stub_dependencies;

/// Isolated temporary repository exercising the production boundary-check binary.
pub struct AuthoritySealingTestRepository {
    pub(super) root: PathBuf,
}

impl AuthoritySealingTestRepository {
    pub fn create(label: &str) -> Self {
        let root = worth_harness::tooling::fixture_path::unique_path(&format!(
            "boundary-check-authority-sealing-{label}"
        ));
        let _ = fs::remove_dir_all(&root);
        Self { root }
    }

    pub fn write_file(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directories");
        }
        fs::write(path, contents).expect("write file");
    }
}
