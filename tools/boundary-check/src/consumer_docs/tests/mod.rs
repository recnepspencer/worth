//! Isolated repositories exercising the production consumer-document entry.
mod ci_ownership;
mod classification;
mod compilation_configuration;
mod debt;
mod links;

use super::configuration::{CONFIG, DEBT, SOURCE};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::{
    fs,
    path::{Path, PathBuf},
};

struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new(text: &str, debt: usize) -> Self {
        let fixture = Self {
            root: worth_harness::tooling::fixture_path::unique_path("consumer-docs"),
        };
        fixture.write("docs/guide.md", text);
        fixture.write(".github/workflows/ci.yml", "jobs:\n  docs:\n    steps:\n      - name: docs\n        run: python scripts/ci/query_gate.py doc\n");
        fixture.write(CONFIG, "documents = ['docs/guide.md']\n[[doctests]]\nmanifest = 'query/Cargo.toml'\npackages = ['example']\n");
        fixture.write(
            "query/Cargo.toml",
            "[workspace]\nmembers = ['crates/example']\n",
        );
        fixture.write(
            "query/crates/example/Cargo.toml",
            "[package]\nname = 'example'\nversion = '0.1.0'\n",
        );
        fixture.write(
            "query/crates/example/src/lib.rs",
            "#![doc = include_str!(\"../../../../docs/guide.md\")]\n",
        );
        fixture.write(
            DEBT,
            &format!("[[documents]]\npath = 'docs/guide.md'\nunclassified = {debt}\n"),
        );
        fixture
    }
    fn write(&self, path: &str, text: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn source(&self, text: &str) {
        self.write(&format!("{SOURCE}/example.rs"), text);
        self.write(&format!("{SOURCE}/lib.rs"), "mod example;\n");
        self.write(
            &format!("{}/Cargo.toml", SOURCE.trim_end_matches("/src")),
            "[package]\nname='worth-query-topology-entry'\nversion='0.1.0'\n",
        );
    }
    fn diagnostics(&self) -> Vec<Diagnostic> {
        super::validate(&self.root)
    }
    fn codes(&self) -> Vec<DiagnosticCode> {
        self.diagnostics().iter().map(Diagnostic::code).collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn configured_repository_documents_have_valid_classifications_and_links() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let diagnostics = super::validate(&root);
    assert!(
        diagnostics.is_empty(),
        "{}",
        crate::diagnostics::render_human(&diagnostics)
    );
}
