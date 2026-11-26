//! Shared RAII Directory Home helper for Query certification courts.
#[path = "directory_home/native_checkpoint.rs"]
mod native_checkpoint;
pub use native_checkpoint::{
    assert_native_checkpoint_fully_unchanged, assert_native_checkpoint_model_unchanged,
    inspect_native_checkpoint_frame,
};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_host::facade::{
    application_installation::WorthQueryProgramApplicationRuntime, WorthQueryApplicationHome,
};

pub fn unique_directory(label: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "worth-dir-{label}-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ))
}

pub fn directory_lease() -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(256 << 20), None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .expect("bounded directory Home lease")
}

pub fn with_directory_bootstrap<T>(
    operation: impl FnOnce(
        &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> T,
) -> T {
    let lease = directory_lease();
    worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request(
        ExecutionRequest::leased(&lease),
        |phase| operation(&phase),
    )
    .expect("directory open bootstrap request")
}

pub fn close_home<Schema: ApplicationSchema, Program: 'static>(
    home: WorthQueryApplicationHome<Schema, Program>,
) {
    let lease = directory_lease();
    home.close(&ExecutionRequest::leased(&lease))
        .unwrap_or_else(|refused| panic!("directory Home close refused: {refused:?}"))
        .reclaim();
}

/// Backup of a closed Directory Home's contents that can be restored in-place
/// into the original directory so its filesystem root identity (inode / file
/// index) remains unchanged across repeated reopens.
pub struct ClosedDirectorySnapshot {
    backup: PathBuf,
}

impl ClosedDirectorySnapshot {
    pub fn capture(source: &Path, label: &str) -> Self {
        let backup = unique_directory(label);
        copy_dir_recursive(source, &backup);
        Self { backup }
    }

    pub fn restore_into(&self, target: &Path) {
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

pub struct DirectoryHomeGuard<Schema: ApplicationSchema, Program: 'static> {
    home: Option<WorthQueryApplicationHome<Schema, Program>>,
    directory: PathBuf,
    cleanup_dir: bool,
}

impl<Schema: ApplicationSchema, Program: 'static> DirectoryHomeGuard<Schema, Program> {
    pub fn wrap(home: WorthQueryApplicationHome<Schema, Program>, directory: PathBuf) -> Self {
        Self {
            home: Some(home),
            directory,
            cleanup_dir: true,
        }
    }

    pub fn home(&self) -> &WorthQueryApplicationHome<Schema, Program> {
        self.home.as_ref().expect("directory Home is open")
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn into_home(mut self) -> (WorthQueryApplicationHome<Schema, Program>, PathBuf) {
        self.cleanup_dir = false;
        let home = self.home.take().expect("directory Home is open");
        (home, self.directory.clone())
    }

    pub fn close_for_reopen(mut self) -> PathBuf {
        self.cleanup_dir = false;
        let home = self.home.take().expect("directory Home is open");
        let lease = directory_lease();
        home.close(&ExecutionRequest::leased(&lease))
            .unwrap_or_else(|refused| panic!("directory Home close refused: {refused:?}"))
            .reclaim();
        self.directory.clone()
    }

    pub fn checkpoint_and_close_for_reopen(mut self) -> PathBuf {
        self.cleanup_dir = false;
        let home = self.home.take().expect("directory Home is open");
        let lease = directory_lease();
        let request = ExecutionRequest::leased(&lease);
        let _ = home
            .checkpoint(&request)
            .unwrap_or_else(|refused| panic!("directory Home checkpoint refused: {refused:?}"));
        home.close(&request)
            .unwrap_or_else(|refused| panic!("directory Home close refused: {refused:?}"))
            .reclaim();
        self.directory.clone()
    }

    /// Publish a real Directory Home checkpoint and extract the exact Native
    /// `WORTH-REL-CHK24` frame referenced by the `home.checkpoint` manifest for
    /// the returned generation.
    pub fn capture_native_checkpoint_frame(&self) -> Vec<u8> {
        let lease = directory_lease();
        let request = ExecutionRequest::leased(&lease);
        let checkpoint = self
            .home()
            .checkpoint(&request)
            .unwrap_or_else(|refused| panic!("directory Home checkpoint refused: {refused:?}"));
        let expected_generation = checkpoint.generation();
        let expected_covered = checkpoint.covered().0;
        let expected_transitions = checkpoint.covered_transitions().0;
        let (covered, transitions, native) =
            native_checkpoint::read_native_checkpoint_frame_for_generation(
                &self.directory,
                expected_generation,
                &request,
            )
            .unwrap_or_else(|| {
                panic!(
                    "published home.checkpoint manifest for generation {expected_generation} not found in Directory Home"
                )
            });
        assert_eq!(
            (covered, transitions),
            (expected_covered, expected_transitions),
            "home.checkpoint generation {expected_generation} coverage mismatch"
        );
        native
    }
}

impl<Schema: ApplicationSchema, Program: 'static> Deref for DirectoryHomeGuard<Schema, Program> {
    type Target = WorthQueryProgramApplicationRuntime<Schema, Program>;

    fn deref(&self) -> &Self::Target {
        self.home
            .as_ref()
            .expect("directory Home is open")
            .runtime()
    }
}

impl<Schema: ApplicationSchema, Program: 'static> Drop for DirectoryHomeGuard<Schema, Program> {
    fn drop(&mut self) {
        if let Some(home) = self.home.take() {
            let lease = directory_lease();
            if let Ok(closed) = home.close(&ExecutionRequest::leased(&lease)) {
                closed.reclaim();
            }
        }
        if self.cleanup_dir {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }
}
