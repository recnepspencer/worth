//! The committed facade documentation debt: names allowed to stay undocumented.

use super::observation::{FacadeDocObservation, NameStatus};
use crate::snapshots::document::{validate_rows, SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// The command that regenerates every governed snapshot, this debt included.
pub(crate) const REGENERATE_COMMAND: &str =
    "cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root . --update-snapshots";

const HEADER: &str = "\
# Facade documentation debt for the stable Query audience facades.
#
# Each entry is a name listed in facades.toml whose definition still lacks
# rustdoc. This file may only shrink: boundary-check reports an undocumented
# name that is absent here (BC8004) and an entry that is already documented or
# no longer exported (BC8005).
#
# To pay an entry down, add a `///` doc comment at the definition boundary-check
# names, then delete the entry.
#
# Regenerating is discouraged. Do it only to defer documentation deliberately,
# and say why in review:
#   cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root . --update-snapshots

";

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct FacadeDocDebtDocument {
    pub(crate) schema_version: u32,
    #[serde(default)]
    pub(crate) facades: Vec<FacadeDocDebtRow>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct FacadeDocDebtRow {
    pub(crate) package: String,
    pub(crate) undocumented: Vec<String>,
}

pub(crate) fn debt_path(root: &Path) -> PathBuf {
    root.join("tools/boundary-check/snapshots/facade-doc-debt.toml")
}

/// The committed debt. An absent file is zero debt, the strictest baseline.
pub(crate) fn load_debt(root: &Path) -> Result<FacadeDocDebtDocument, String> {
    let path = debt_path(root);
    if !path.exists() {
        return Ok(FacadeDocDebtDocument {
            schema_version: SCHEMA_VERSION,
            facades: Vec::new(),
        });
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let document: FacadeDocDebtDocument =
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    validate_rows(
        document.schema_version,
        document
            .facades
            .iter()
            .map(|row| (row.package.as_str(), row.undocumented.as_slice())),
    )
    .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(document)
}

/// The debt that exactly matches `observation`; unresolved names block it.
pub(crate) fn debt_from_observation(
    observation: &FacadeDocObservation,
) -> Result<FacadeDocDebtDocument, String> {
    let mut facades = Vec::new();
    for (package, names) in &observation.facades {
        let mut undocumented = Vec::new();
        for (name, status) in names {
            match status {
                NameStatus::Undocumented(_) => undocumented.push(name.clone()),
                NameStatus::Unresolved(reason) => {
                    return Err(format!(
                        "cannot record facade doc debt: {package}::{name} does not resolve to a definition ({reason})"
                    ))
                }
                NameStatus::Documented | NameStatus::OutsideWorkspace => {}
            }
        }
        if !undocumented.is_empty() {
            facades.push(FacadeDocDebtRow {
                package: package.clone(),
                undocumented,
            });
        }
    }
    Ok(FacadeDocDebtDocument {
        schema_version: SCHEMA_VERSION,
        facades,
    })
}

pub(crate) fn render_debt(document: &FacadeDocDebtDocument) -> Result<String, String> {
    let body = toml::to_string_pretty(document).map_err(|error| error.to_string())?;
    Ok(format!("{HEADER}{body}"))
}

impl FacadeDocDebtDocument {
    pub(crate) fn contains(&self, package: &str, name: &str) -> bool {
        self.facades
            .iter()
            .any(|row| row.package == package && row.undocumented.iter().any(|entry| entry == name))
    }
}
