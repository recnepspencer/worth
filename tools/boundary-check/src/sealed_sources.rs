//! One source traversal for the two named reach seals.
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::{fs, path::Path};
pub(crate) fn validate(root: &Path) -> Vec<Diagnostic> {
    let (mut diagnostics, doors) = crate::control_seal::load(root);
    let files = match crate::governed_sources::files_under(root) {
        Ok(files) => files,
        Err(error) => {
            diagnostics.push(failure(".", error));
            return diagnostics;
        }
    };
    for file in files
        .into_iter()
        .filter(|file| file.extension().is_some_and(|ext| ext == "rs"))
    {
        let path = file
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        match fs::read_to_string(&file) {
            Ok(source) => {
                diagnostics.extend(crate::certification_construction::source_references(
                    &path, &source,
                ));
                diagnostics.extend(crate::control_seal::references(&path, &source, &doors));
            }
            Err(error) => diagnostics.push(failure(&path, error.to_string())),
        }
    }
    diagnostics
}
fn failure(path: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Bc7012ControlSeal, path, message)
}
