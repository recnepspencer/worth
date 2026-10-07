use super::baseline::{compare_exact_sets, dag_path, facade_path};
use super::commit::commit_snapshots;
use super::crate_dag::crate_dag_document;
use super::document::{CrateDagDocument, FacadeDocument};
use super::facade_surface_observation::{
    observe_facade_document, ConfiguredFacadeSurface, ObservedFacadeExports,
};
use crate::diagnostics::Diagnostic;
use crate::facade_docs::{
    debt_from_observation, debt_path, facade_doc_diagnostics, observe_facade_docs, render_debt,
    FacadeDocObservation,
};
use crate::manifest_types::Road1Package;
use std::path::{Path, PathBuf};

pub(crate) struct ConstitutionSnapshots {
    dag: CrateDagDocument,
    facades: FacadeDocument,
    facade_docs: FacadeDocObservation,
}

impl ConstitutionSnapshots {
    pub(crate) fn observe(
        root: &Path,
        packages: &[Road1Package],
        dag_only_packages: &[Road1Package],
        configured_surfaces: &[ConfiguredFacadeSurface],
    ) -> Result<Self, String> {
        let facades = observe_facade_document(packages, configured_surfaces)?;
        let facade_docs = observe_facade_docs(root, packages, &facades)?;
        let mut dag_packages = packages.to_vec();
        dag_packages.extend_from_slice(dag_only_packages);
        Ok(Self {
            dag: crate_dag_document(&dag_packages),
            facades,
            facade_docs,
        })
    }

    pub(crate) fn check(&self, root: &Path) -> Vec<Diagnostic> {
        let mut diagnostics = compare_exact_sets(root, &self.dag, &self.facades);
        diagnostics.extend(facade_doc_diagnostics(root, &self.facade_docs));
        diagnostics
    }

    pub(crate) fn write(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let debt = debt_from_observation(&self.facade_docs)?;
        commit_snapshots(&[
            (dag_path(root), render(&self.dag)?),
            (facade_path(root), render(&self.facades)?),
            (debt_path(root), render_debt(&debt)?),
        ])
    }

    pub(crate) fn observed_facade_exports(&self) -> ObservedFacadeExports {
        ObservedFacadeExports::from_document(&self.facades)
    }
}

fn render<T: serde::Serialize>(document: &T) -> Result<String, String> {
    toml::to_string_pretty(document).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::DiagnosticCode;
    use crate::snapshots::document::{DependencyRow, FacadeRow, SCHEMA_VERSION};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("boundary-snapshot-{id}"))
    }

    fn candidate() -> ConstitutionSnapshots {
        ConstitutionSnapshots {
            dag: CrateDagDocument {
                schema_version: SCHEMA_VERSION,
                packages: vec![DependencyRow {
                    package: "worth-schema-core".into(),
                    dependencies: vec![],
                }],
            },
            facades: FacadeDocument {
                schema_version: SCHEMA_VERSION,
                facades: vec![FacadeRow {
                    package: "worth-schema-core".into(),
                    exports: vec!["Identity".into()],
                }],
            },
            facade_docs: FacadeDocObservation::default(),
        }
    }

    #[test]
    fn dag_only_store_package_does_not_create_a_facade_or_doc_surface() {
        let package = Road1Package {
            name: "worth-store".into(),
            dependencies: vec!["worth-store-blob-chunks".into()],
            manifest_path: "unused/Cargo.toml".into(),
        };
        let observed =
            ConstitutionSnapshots::observe(Path::new("."), &[], &[package], &[]).unwrap();
        assert_eq!(observed.dag.packages.len(), 1);
        assert_eq!(observed.dag.packages[0].package, "worth-store");
        assert!(observed.facades.facades.is_empty());
        assert!(observed.facade_docs.facades.is_empty());
    }

    #[test]
    fn regeneration_is_byte_identical_and_check_is_exact() {
        let root = root();
        let baseline = candidate();
        baseline.write(&root).unwrap();
        let first_dag =
            fs::read(root.join("tools/boundary-check/snapshots/crate-dag.toml")).unwrap();
        let first_facades =
            fs::read(root.join("tools/boundary-check/snapshots/facades.toml")).unwrap();
        baseline.write(&root).unwrap();
        assert_eq!(
            first_dag,
            fs::read(root.join("tools/boundary-check/snapshots/crate-dag.toml")).unwrap()
        );
        assert_eq!(
            first_facades,
            fs::read(root.join("tools/boundary-check/snapshots/facades.toml")).unwrap()
        );
        assert!(baseline.check(&root).is_empty());

        let mut widened = candidate();
        widened.dag.packages[0]
            .dependencies
            .push("worth-query-decl".into());
        widened.facades.facades[0].exports.push("Name".into());
        let codes = widened
            .check(&root)
            .into_iter()
            .map(|diagnostic| diagnostic.code())
            .collect::<Vec<_>>();
        assert!(codes.contains(&DiagnosticCode::Bc8003CrateDagSnapshotDrift));
        assert!(codes.contains(&DiagnosticCode::Bc8002FacadeSnapshotDrift));

        baseline.write(&root).unwrap();
        let empty = ConstitutionSnapshots {
            dag: CrateDagDocument {
                schema_version: SCHEMA_VERSION,
                packages: vec![],
            },
            facades: FacadeDocument {
                schema_version: SCHEMA_VERSION,
                facades: vec![],
            },
            facade_docs: FacadeDocObservation::default(),
        };
        assert!(!empty.check(&root).is_empty());
    }
}
