//! Fixture repositories for the facade documentation ratchet.

mod ratchet_cases;
mod resolution_cases;

use super::observation::{observe_facade_docs, FacadeDocObservation, NameStatus};
use super::{debt_from_observation, debt_path, facade_doc_diagnostics, render_debt};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::manifest_types::Road1Package;
use crate::snapshots::document::{FacadeDocument, FacadeRow, SCHEMA_VERSION};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

/// A throwaway repository holding a Cargo workspace of fixture crates.
struct Fixture {
    root: PathBuf,
    packages: Vec<Road1Package>,
}

impl Fixture {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "boundary-facade-docs-{nanos}-{}",
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .unwrap();
        Self {
            root,
            packages: Vec::new(),
        }
    }

    /// Add crate `package` with `files` (path under `src/`, contents) and
    /// path dependencies on the named fixture crates.
    fn crate_with(&mut self, package: &str, dependencies: &[&str], files: &[(&str, &str)]) {
        let crate_root = self.root.join("crates").join(package);
        let mut manifest =
            format!("[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n");
        for dependency in dependencies {
            manifest.push_str(&format!(
                "{dependency} = {{ path = \"../{dependency}\" }}\n"
            ));
        }
        manifest.push_str("serde = \"1\"\n");
        self.write(&crate_root.join("Cargo.toml"), &manifest);
        for (path, contents) in files {
            self.write(&crate_root.join("src").join(path), contents);
        }
        self.packages.push(Road1Package {
            name: package.to_owned(),
            dependencies: dependencies.iter().map(|name| (*name).to_owned()).collect(),
            manifest_path: crate_root.join("Cargo.toml").display().to_string(),
        });
    }

    fn write(&self, path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn observe(&self, rows: &[(&str, &[&str])]) -> FacadeDocObservation {
        let facades = FacadeDocument {
            schema_version: SCHEMA_VERSION,
            facades: rows
                .iter()
                .map(|(package, exports)| FacadeRow {
                    package: (*package).to_owned(),
                    exports: exports.iter().map(|name| (*name).to_owned()).collect(),
                })
                .collect(),
        };
        observe_facade_docs(&self.root, &self.packages, &facades).unwrap()
    }

    /// Record the current undocumented names as debt, as `--update-snapshots` does.
    fn record_debt(&self, observation: &FacadeDocObservation) {
        let debt = debt_from_observation(observation).unwrap();
        self.write(&debt_path(&self.root), &render_debt(&debt).unwrap());
    }

    fn write_debt(&self, contents: &str) {
        self.write(&debt_path(&self.root), contents);
    }

    fn diagnostics(&self, observation: &FacadeDocObservation) -> Vec<Diagnostic> {
        facade_doc_diagnostics(&self.root, observation)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn codes(diagnostics: &[Diagnostic]) -> Vec<DiagnosticCode> {
    diagnostics.iter().map(Diagnostic::code).collect()
}

fn site_of(observation: &FacadeDocObservation, package: &str, name: &str) -> Vec<String> {
    match observation.status(package, name) {
        Some(NameStatus::Undocumented(missing)) => {
            missing.iter().map(|entry| entry.site.clone()).collect()
        }
        other => panic!("{package}::{name} is not undocumented: {other:?}"),
    }
}
