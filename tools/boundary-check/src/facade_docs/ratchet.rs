//! Compare observed facade documentation with the committed debt so the debt
//! only shrinks.

use super::debt_snapshot::{debt_path, load_debt, FacadeDocDebtDocument, REGENERATE_COMMAND};
use super::doc_presence::MissingDoc;
use super::observation::{FacadeDocObservation, NameStatus};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::path::Path;

const DEBT_FILE: &str = "tools/boundary-check/snapshots/facade-doc-debt.toml";

/// BC8004 for undocumented names outside the debt, BC8005 for paid debt.
pub(crate) fn facade_doc_diagnostics(
    root: &Path,
    observation: &FacadeDocObservation,
) -> Vec<Diagnostic> {
    let debt = match load_debt(root) {
        Ok(debt) => debt,
        Err(error) => {
            return vec![Diagnostic::with_legal_home(
                DiagnosticCode::Bc8001SnapshotBaseline,
                debt_path(root).display().to_string(),
                error,
                format!("{DEBT_FILE}; restore the committed debt file"),
            )]
        }
    };
    let mut diagnostics = Vec::new();
    for (package, names) in &observation.facades {
        for (name, status) in names {
            if !debt.contains(package, name) {
                diagnostics.extend(missing_without_debt(package, name, status));
            }
        }
    }
    diagnostics.extend(stale_debt(observation, &debt));
    diagnostics
}

fn missing_without_debt(package: &str, name: &str, status: &NameStatus) -> Vec<Diagnostic> {
    let subject = format!("{package}::{name}");
    match status {
        NameStatus::Documented | NameStatus::OutsideWorkspace => Vec::new(),
        NameStatus::Undocumented(missing) => missing
            .iter()
            .map(|site| undocumented(&subject, site))
            .collect(),
        NameStatus::Unresolved(reason) => vec![Diagnostic::with_legal_home(
            DiagnosticCode::Bc8004FacadeDocMissing,
            subject,
            format!(
                "stable facade name cannot be resolved to its definition ({reason}), so its rustdoc cannot be checked"
            ),
            "tools/boundary-check/src/facade_docs/resolution/mod.rs; teach the resolver this export shape or export the item through a plain `pub use` path",
        )],
    }
}

fn undocumented(subject: &str, missing: &MissingDoc) -> Diagnostic {
    let MissingDoc { kind, site } = missing;
    Diagnostic::with_legal_home(
        DiagnosticCode::Bc8004FacadeDocMissing,
        subject,
        format!(
            "stable facade {kind} has no rustdoc; add a /// doc comment at {site}"
        ),
        format!(
            "{site}; add a `///` doc comment there. Only to defer deliberately (discouraged): run `{REGENERATE_COMMAND}` to record it in {DEBT_FILE}"
        ),
    )
}

fn stale_debt(observation: &FacadeDocObservation, debt: &FacadeDocDebtDocument) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for row in &debt.facades {
        for name in &row.undocumented {
            let reason = match observation.status(&row.package, name) {
                None => "is no longer exported",
                Some(NameStatus::Documented) => "is now documented",
                Some(NameStatus::OutsideWorkspace) => "is defined outside the workspace",
                Some(NameStatus::Undocumented(_) | NameStatus::Unresolved(_)) => continue,
            };
            diagnostics.push(Diagnostic::with_legal_home(
                DiagnosticCode::Bc8005FacadeDocDebtStale,
                format!("{}::{name}", row.package),
                format!(
                    "facade doc debt entry {reason}; delete it from {DEBT_FILE} so the debt only shrinks"
                ),
                format!(
                    "{DEBT_FILE}; delete `{name}` from the `{}` row",
                    row.package
                ),
            ));
        }
    }
    diagnostics
}
