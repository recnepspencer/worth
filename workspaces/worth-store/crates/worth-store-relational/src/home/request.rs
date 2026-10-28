use std::path::{Path, PathBuf};

use worth_relational::facade::runtime::RelationalRuntimeBuilder;

use super::RelationalStoreHomeOpenRefusal;

/// Where a home lives and the runtime it serves.
///
/// The builder declares everything about the runtime except its durable
/// medium, which the home attaches. Opening an existing home with a builder
/// whose runtime name or profile differs from the home's refuses, and so does
/// one whose declared schema registry is not the schema the home's recovered
/// main head records (`Readmission` with
/// `RecoveryFailureClass::DeclaredSchemaMismatch`). A home with no history
/// serves the schema its first runtime declares.
#[derive(Debug, Clone)]
pub struct RelationalStoreHomeRequest {
    location: PathBuf,
    runtime: RelationalRuntimeBuilder,
}

impl RelationalStoreHomeRequest {
    pub fn new(location: impl Into<PathBuf>, runtime: RelationalRuntimeBuilder) -> Self {
        Self {
            location: location.into(),
            runtime,
        }
    }

    pub fn location(&self) -> &Path {
        &self.location
    }

    pub(super) fn into_parts(self) -> (PathBuf, RelationalRuntimeBuilder) {
        (self.location, self.runtime)
    }
}

/// A location is unused when it does not exist or is an empty directory; only
/// then may a new home be created there. It is created if absent.
pub(super) fn location_is_unused(location: &Path) -> Result<bool, RelationalStoreHomeOpenRefusal> {
    let location_refusal = |error: std::io::Error| RelationalStoreHomeOpenRefusal::Location {
        kind: error.kind(),
        detail: error.to_string(),
    };
    if !location.exists() {
        std::fs::create_dir_all(location).map_err(location_refusal)?;
        return Ok(true);
    }
    let mut entries = std::fs::read_dir(location).map_err(location_refusal)?;
    Ok(entries.next().is_none())
}
