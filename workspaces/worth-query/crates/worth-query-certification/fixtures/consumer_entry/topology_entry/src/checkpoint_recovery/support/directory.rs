//! Local Directory Home path, policy, and snapshot helpers for topology_entry courts.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::application_contribution::{
    with_bootstrap_advancement, WorthQueryBootstrapAdvancementPhase,
};

pub(in crate::checkpoint_recovery) fn unique_directory(label: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "worth-topo-{label}-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ))
}

pub(in crate::checkpoint_recovery) fn directory_policy() -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(256 << 20), None),
    )
}

pub(in crate::checkpoint_recovery) fn with_directory_bootstrap<T>(
    operation: impl FnOnce(&WorthQueryBootstrapAdvancementPhase<'_>) -> T,
) -> T {
    with_bootstrap_advancement(directory_policy(), |phase| operation(&phase))
        .expect("topology_entry directory advancement request")
}

/// Backup of a closed Directory Home's contents that can be restored in-place
/// into the original directory so its filesystem root identity remains unchanged.
pub(in crate::checkpoint_recovery) struct ClosedDirectorySnapshot {
    backup: PathBuf,
}

impl ClosedDirectorySnapshot {
    pub(in crate::checkpoint_recovery) fn capture(source: &Path, label: &str) -> Self {
        let backup = unique_directory(label);
        copy_dir_recursive(source, &backup);
        Self { backup }
    }

    pub(in crate::checkpoint_recovery) fn restore_into(&self, target: &Path) {
        clear_dir_contents(target);
        copy_dir_recursive(&self.backup, target);
    }
}

impl Drop for ClosedDirectorySnapshot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.backup);
    }
}

fn clear_dir_contents(dir: &Path) {
    for entry in std::fs::read_dir(dir).expect("read target home directory") {
        let entry = entry.expect("target directory entry");
        let path = entry.path();
        if entry.file_type().expect("target entry type").is_dir() {
            std::fs::remove_dir_all(&path).expect("remove target subdirectory");
        } else {
            std::fs::remove_file(&path).expect("remove target file");
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("create copied home directory");
    for entry in std::fs::read_dir(src).expect("read source home directory") {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if entry.file_type().expect("entry type").is_dir() {
            copy_dir_recursive(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("copy home file");
        }
    }
}
