//! Independently compiled executables supplied by the test runner before tests.

use std::path::PathBuf;

use serde::Deserialize;

pub const OFFLINE_ROOT_OBSERVER: IndependentExecutable = IndependentExecutable::new(
    "worth-store-offline-integrity-observer",
    "physical_store_integrity_observer",
);

pub const OFFLINE_RECOVERY_OBSERVER: IndependentExecutable = IndependentExecutable::new(
    "worth-store-offline-verifier",
    "physical_store_offline_observer",
);

// These children are killed, never power-cut. Keep the existing volatile-sync
// graph and certification admission; moving the build must not alter their I/O.
pub const FRESH_PROCESS_RECOVERY: IndependentExecutable =
    IndependentExecutable::new("worth-store-recovery-runtime", "physical_store_recover")
        .with_features(
            "worth-store-recovery-runtime/certification-test-authority,\
             worth-store/volatile-sync-for-tests",
        );

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndependentExecutable {
    package: &'static str,
    binary: &'static str,
    features: &'static str,
}

#[derive(Debug)]
pub enum PrebuiltExecutableRefusal {
    MissingManifest,
    ManifestRead(std::io::Error),
    InvalidManifest(serde_json::Error),
    UnsupportedManifest(u32),
    MissingExecutable {
        package: &'static str,
        binary: &'static str,
    },
    MissingFile(PathBuf),
}

#[derive(Deserialize)]
struct PrebuiltExecutables {
    schema: u32,
    executables: Vec<PrebuiltExecutable>,
}

#[derive(Deserialize)]
struct PrebuiltExecutable {
    package: String,
    binary: String,
    features: Vec<String>,
    path: PathBuf,
}

impl IndependentExecutable {
    pub const fn new(package: &'static str, binary: &'static str) -> Self {
        Self {
            package,
            binary,
            features: "",
        }
    }

    pub const fn with_features(self, features: &'static str) -> Self {
        Self { features, ..self }
    }

    /// Resolve a runner-owned artifact. A court never invokes a compiler.
    pub fn path(&self) -> Result<PathBuf, PrebuiltExecutableRefusal> {
        let manifest = std::env::var_os("WORTH_PREBUILT_EXECUTABLES")
            .ok_or(PrebuiltExecutableRefusal::MissingManifest)?;
        let bytes = std::fs::read(manifest).map_err(PrebuiltExecutableRefusal::ManifestRead)?;
        let manifest: PrebuiltExecutables =
            serde_json::from_slice(&bytes).map_err(PrebuiltExecutableRefusal::InvalidManifest)?;
        if manifest.schema != 1 {
            return Err(PrebuiltExecutableRefusal::UnsupportedManifest(
                manifest.schema,
            ));
        }
        let expected = self.normalized_features();
        let executable = manifest
            .executables
            .into_iter()
            .find(|executable| {
                executable.package == self.package
                    && executable.binary == self.binary
                    && executable.features == expected
            })
            .ok_or(PrebuiltExecutableRefusal::MissingExecutable {
                package: self.package,
                binary: self.binary,
            })?;
        if !executable.path.is_file() {
            return Err(PrebuiltExecutableRefusal::MissingFile(executable.path));
        }
        Ok(executable.path)
    }

    fn normalized_features(&self) -> Vec<String> {
        let mut features = self
            .features
            .split(',')
            .map(str::trim)
            .filter(|feature| !feature.is_empty())
            .map(|feature| {
                if feature.contains('/') {
                    feature.to_owned()
                } else {
                    format!("{}/{feature}", self.package)
                }
            })
            .collect::<Vec<_>>();
        features.sort();
        features.dedup();
        features
    }
}
