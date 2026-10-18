//! Exact per-document unclassified counts; never automatically regenerated.
use super::configuration::DEBT;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Debt {
    documents: Vec<Row>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    path: String,
    unclassified: usize,
}

pub(super) fn compare(root: &Path, observed: &BTreeMap<String, usize>) -> Vec<Diagnostic> {
    let result = load(root);
    let debt = match result {
        Ok(debt) => debt,
        Err(error) => {
            return vec![Diagnostic::new(
                DiagnosticCode::Bc8001SnapshotBaseline,
                DEBT,
                error,
            )]
        }
    };
    let mut diagnostics = Vec::new();
    for (path, count) in observed {
        match debt.get(path) {
            None => diagnostics.push(Diagnostic::new(DiagnosticCode::Bc8001SnapshotBaseline, path,
                "consumer debt must have one row for every configured document")),
            Some(allowed) if count > allowed => diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc8006ConsumerFenceDebtExceeded, path,
                format!("{count} unclassified Rust fences exceed debt {allowed}; classify the new fences"))),
            Some(allowed) if count < allowed => diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc8007ConsumerFenceDebtStale, path,
                format!("{count} unclassified Rust fences remain, below debt {allowed}; lower the debt row"))),
            _ => {}
        }
    }
    for path in debt.keys().filter(|path| !observed.contains_key(*path)) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc8007ConsumerFenceDebtStale,
            path,
            "debt document is no longer configured; restore coverage or remove the stale row",
        ));
    }
    diagnostics
}

fn load(root: &Path) -> Result<BTreeMap<String, usize>, String> {
    let text = crate::source_documents::read_to_string(&root.join(DEBT))
        .map_err(|error| error.to_string())?;
    let debt: Debt = toml::from_str(&text).map_err(|error| error.to_string())?;
    let mut rows = BTreeMap::new();
    for row in debt.documents {
        if rows.insert(row.path.clone(), row.unclassified).is_some() {
            return Err(format!("duplicate consumer debt row: {}", row.path));
        }
    }
    Ok(rows)
}
