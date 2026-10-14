//! Leaf-facade contract for configured Query audience crates.
//!
//! Validates that each matrix audience package under the configured Query
//! workspace is a zero-behavior re-export leaf over the configured engine: one
//! direct engine dependency, facade-only lib surface, and re-export-only facade
//! module. The optional certification package is a cold leaf over the engine.

mod dependencies;
mod ordinary_entry;
mod surface;

use crate::config::QueryAudienceContract;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use dependencies::{validate_authority_dependencies, validate_cold_certification_leaf};
use std::path::Path;
use surface::{validate_facade_only_lib, validate_reexport_only_facade};

pub(crate) fn validate_query_audience_facades(
    root: &Path,
    contract: &QueryAudienceContract,
) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(ordinary_entry::validate(root, &contract.workspace)?);
    let audience_packages: Vec<&str> = contract
        .audiences
        .iter()
        .map(|audience| audience.package.as_str())
        .collect();
    let query_crates_root = root.join(&contract.workspace).join("crates");

    for audience in &contract.audiences {
        let crate_root = query_crates_root.join(&audience.package);
        let relative = relative_to_root(root, &crate_root);
        if !crate_root.is_dir() {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc3003QueryAudienceFacadeContract,
                relative,
                format!(
                    "configured Query audience facade `{}` is missing under `{}/crates`",
                    audience.package, contract.workspace
                ),
            ));
            continue;
        }

        let authority_packages = if audience.authority_packages.is_empty() {
            vec![contract.engine_package.clone()]
        } else {
            audience.authority_packages.clone()
        };
        diagnostics.extend(validate_authority_dependencies(
            &crate_root,
            &relative,
            &authority_packages,
            &audience_packages,
        )?);
        diagnostics.extend(validate_facade_only_lib(
            &crate_root.join("src/lib.rs"),
            &relative,
        )?);
        diagnostics.extend(validate_reexport_only_facade(
            &crate_root.join("src/facade.rs"),
            &relative,
            &authority_packages,
            &audience_packages,
        )?);
    }

    if let Some(certification_package) = &contract.certification_package {
        diagnostics.extend(validate_cold_certification_leaf(
            root,
            &query_crates_root,
            certification_package,
            contract,
        )?);
    }

    Ok(diagnostics)
}

pub(super) fn relative_to_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
