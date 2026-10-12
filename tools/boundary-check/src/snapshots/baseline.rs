use super::document::{validate_rows, CrateDagDocument, FacadeDocument};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::path::{Path, PathBuf};

pub(super) fn dag_path(root: &Path) -> PathBuf {
    root.join("tools/boundary-check/snapshots/crate-dag.toml")
}

pub(super) fn facade_path(root: &Path) -> PathBuf {
    root.join("tools/boundary-check/snapshots/facades.toml")
}

pub(crate) fn validate_committed_dag_baseline(root: &Path) -> Vec<Diagnostic> {
    let path = dag_path(root);
    load_dag(&path)
        .err()
        .map(|error| vec![baseline_diagnostic(&path, error)])
        .unwrap_or_default()
}

pub(super) fn compare_exact_sets(
    root: &Path,
    dag: &CrateDagDocument,
    facades: &FacadeDocument,
) -> Vec<Diagnostic> {
    let dag_path = dag_path(root);
    let facade_path = facade_path(root);
    let mut diagnostics = Vec::new();
    match load_dag(&dag_path) {
        Ok(expected) if expected != *dag => diagnostics.push(drift(
            DiagnosticCode::Bc8003CrateDagSnapshotDrift,
            &dag_path,
            &expected,
            dag,
        )),
        Err(error) => diagnostics.push(baseline_diagnostic(&dag_path, error)),
        _ => {}
    }
    match load_facades(&facade_path) {
        Ok(expected) if expected != *facades => diagnostics.push(drift(
            DiagnosticCode::Bc8002FacadeSnapshotDrift,
            &facade_path,
            &expected,
            facades,
        )),
        Err(error) => diagnostics.push(baseline_diagnostic(&facade_path, error)),
        _ => {}
    }
    diagnostics
}

pub(super) fn load_facades(path: &Path) -> Result<FacadeDocument, String> {
    let value: FacadeDocument = parse(path)?;
    validate_rows(
        value.schema_version,
        value
            .facades
            .iter()
            .map(|row| (row.package.as_str(), row.exports.as_slice())),
    )?;
    Ok(value)
}

fn load_dag(path: &Path) -> Result<CrateDagDocument, String> {
    let value: CrateDagDocument = parse(path)?;
    validate_rows(
        value.schema_version,
        value
            .packages
            .iter()
            .map(|row| (row.package.as_str(), row.dependencies.as_slice())),
    )?;
    Ok(value)
}

fn parse<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = crate::source_documents::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

pub(super) fn baseline_diagnostic(path: &Path, message: String) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc8001SnapshotBaseline,
        path.display().to_string(),
        message,
    )
}

fn drift<T: std::fmt::Debug>(code: DiagnosticCode, path: &Path, old: &T, new: &T) -> Diagnostic {
    Diagnostic::new(code, path.display().to_string(), format!("exact set changed; regenerate explicitly with --update-snapshots\nexpected: {old:#?}\nobserved: {new:#?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_rows_and_values_are_rejected() {
        assert!(validate_rows(1, [("a", &["x".into(), "x".into()][..])]).is_err());
        assert!(validate_rows(1, [("a", &[][..]), ("a", &[][..])]).is_err());
    }
}
