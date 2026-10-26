//! Store executables that fresh-process tests run as independent processes.
//!
//! A process test must exercise a binary Cargo built on its own, never the
//! test's in-process image. The test owns that prerequisite: the first use in
//! a test process builds the declared package binary with its declared
//! features, so `cargo test` of any package, nextest, and the Store runner all
//! reach the same executable without an environment contract.
//!
//! The build uses its own target directory beside the test's, so it never
//! replaces an executable the outer build produced. Each built executable is
//! copied to a file named by its digest; a running copy is never replaced, so
//! concurrent test processes on Windows cannot lock each other out.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::compiler_boundary::bounded_process;

const BUILD_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const BUILD_OUTPUT_LIMIT: usize = 16 * 1024 * 1024;

static BUILT: Mutex<BTreeMap<IndependentExecutable, PathBuf>> = Mutex::new(BTreeMap::new());

/// The C.9 offline root observer: reads inert Store artifacts and links no
/// Store runtime.
pub const OFFLINE_ROOT_OBSERVER: IndependentExecutable = IndependentExecutable::new(
    "worth-store-offline-integrity-observer",
    "physical_store_integrity_observer",
);

/// The offline verifier's observer of a closed Store's recovery evidence.
pub const OFFLINE_RECOVERY_OBSERVER: IndependentExecutable = IndependentExecutable::new(
    "worth-store-offline-verifier",
    "physical_store_offline_observer",
);

/// The standalone C8 recovery executable. Test children are killed, never
/// power-cut, so recovery skips the OS flush exactly as the in-process suites
/// do; certification profiles stay available to the recovery scenarios.
pub const FRESH_PROCESS_RECOVERY: IndependentExecutable =
    IndependentExecutable::new("worth-store-recovery-runtime", "physical_store_recover")
        .with_features(
            "worth-store-recovery-runtime/certification-test-authority,\
     worth-store/volatile-sync-for-tests",
        );

/// One Store workspace binary, built without default features plus exactly
/// the declared feature list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndependentExecutable {
    package: &'static str,
    binary: &'static str,
    features: &'static str,
}

impl IndependentExecutable {
    pub const fn new(package: &'static str, binary: &'static str) -> Self {
        Self {
            package,
            binary,
            features: "",
        }
    }

    /// Cargo `--features` syntax, so a dependency's feature is `crate/feature`.
    pub const fn with_features(self, features: &'static str) -> Self {
        Self { features, ..self }
    }

    /// The executable's path, building it on first use in this process.
    /// A failed build fails the calling test with Cargo's diagnostics.
    pub fn path(&self) -> PathBuf {
        let mut built = BUILT
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(path) = built.get(self) {
            return path.clone();
        }
        let path = self
            .build()
            .unwrap_or_else(|failure| panic!("independent executable {self:?}: {failure}"));
        built.insert(*self, path.clone());
        path
    }

    fn build(&self) -> Result<PathBuf, String> {
        let layout = TargetLayout::of_current_test()?;
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
        command
            .current_dir(&workspace)
            .env("CARGO_TARGET_DIR", &layout.build_root)
            .args([
                "build",
                "--locked",
                "--message-format=json-render-diagnostics",
            ])
            .arg("--manifest-path")
            .arg(workspace.join("Cargo.toml"))
            .args(["--profile", &layout.profile, "--no-default-features"])
            .args(["-p", self.package, "--bin", self.binary]);
        if !self.features.is_empty() {
            command.args(["--features", self.features]);
        }
        let output = bounded_process::run(&mut command, BUILD_TIMEOUT, BUILD_OUTPUT_LIMIT)?;
        if output.timed_out || !output.status.success() {
            return Err(format!(
                "cargo build {} (timed out: {})\n{}",
                output.status,
                output.timed_out,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let built = built_executable(&output.stdout, self.binary)?;
        retained_copy(&built, &layout.build_root.join("retained"), self.binary)
    }
}

/// Where the running test's Cargo build lives: `<target>/<profile>/deps`.
struct TargetLayout {
    build_root: PathBuf,
    profile: String,
}

impl TargetLayout {
    fn of_current_test() -> Result<Self, String> {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let deps = executable.parent().filter(|deps| deps.ends_with("deps"));
        let profile_directory = deps.and_then(Path::parent);
        let target = profile_directory.and_then(Path::parent);
        let (Some(profile_directory), Some(target)) = (profile_directory, target) else {
            return Err(format!(
                "{} is not a Cargo test executable under <target>/<profile>/deps",
                executable.display()
            ));
        };
        let profile = match profile_directory.file_name().and_then(|name| name.to_str()) {
            Some("debug") => "dev".to_owned(),
            Some(profile) => profile.to_owned(),
            None => return Err(format!("unnamed profile {}", profile_directory.display())),
        };
        Ok(Self {
            build_root: target.join("independent-executables"),
            profile,
        })
    }
}

fn built_executable(messages: &[u8], binary: &str) -> Result<PathBuf, String> {
    let messages = String::from_utf8_lossy(messages);
    messages
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|message| message["reason"] == "compiler-artifact")
        .filter(|message| message["target"]["name"] == binary)
        .find_map(|message| message["executable"].as_str().map(PathBuf::from))
        .ok_or_else(|| format!("Cargo reported no executable for `{binary}`"))
}

fn retained_copy(built: &Path, directory: &Path, binary: &str) -> Result<PathBuf, String> {
    let bytes =
        std::fs::read(built).map_err(|error| format!("read {}: {error}", built.display()))?;
    let digest = Sha256::digest(&bytes);
    let name = format!(
        "{binary}-{}{}",
        digest[..8]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        std::env::consts::EXE_SUFFIX
    );
    let retained = directory.join(name);
    if retained.is_file() {
        return Ok(retained);
    }
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("create {}: {error}", directory.display()))?;
    let partial = directory.join(format!("{binary}.{}.partial", std::process::id()));
    std::fs::write(&partial, &bytes)
        .map_err(|error| format!("write {}: {error}", partial.display()))?;
    let permissions = std::fs::metadata(built)
        .map_err(|error| format!("stat {}: {error}", built.display()))?
        .permissions();
    std::fs::set_permissions(&partial, permissions)
        .map_err(|error| format!("permissions {}: {error}", partial.display()))?;
    match std::fs::rename(&partial, &retained) {
        Ok(()) => Ok(retained),
        // Another process published the same digest first; its file is ours.
        Err(_) if retained.is_file() => {
            let _ = std::fs::remove_file(&partial);
            Ok(retained)
        }
        Err(error) => Err(format!("publish {}: {error}", retained.display())),
    }
}
