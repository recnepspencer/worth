//! Deterministic Rust-source ceilings for compiler units, never wall time.
use super::failure;
use crate::{diagnostics::Diagnostic, source_rules::GovernedCrate};
use std::path::Path;

pub(super) const CEILING: usize = 15_000;

pub(super) fn rust_lines(directory: &Path) -> Result<usize, String> {
    let mut count = 0;
    for entry in std::fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            if path
                .file_name()
                .is_some_and(|n| n != "target" && n != ".git")
            {
                count += rust_lines(&path)?;
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            count += crate::source_documents::read_to_string(&path)
                .map_err(|e| e.to_string())?
                .lines()
                .count();
        }
    }
    Ok(count)
}

pub(super) fn validate(owner: &GovernedCrate) -> Vec<Diagnostic> {
    match rust_lines(&owner.crate_root) {
        Ok(lines) if lines <= CEILING => Vec::new(),
        Ok(lines) => vec![failure(&owner.relative_crate_root, format!("court/testkit has {lines} Rust lines; ceiling is {CEILING}; split at the next area"))],
        Err(error) => vec![failure(&owner.relative_crate_root, error)],
    }
}
