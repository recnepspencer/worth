//! Source-level constitutional passes over governed Rust crates.
//!
//! Phase 3 owns authority sealing. Later phases (import shape, signature leak,
//! re-export fence) share this facade without inheriting sealing classification.

mod analysis;
mod law_substrates;

use crate::config::{LawSubstrateConfig, NamingConfig, QueryAudienceContract, SubworkspaceConfig};
use crate::diagnostics::Diagnostic;
use crate::snapshots::FacadeVocabularyAuthority;
use std::path::Path;

pub(crate) use analysis::{
    enforce_raw_geometry_denials, enforce_truth_type_denials, observe_compiled_library_surface,
};
pub(crate) use law_substrates::{illegal_law_substrate_edge, is_legal_law_substrate_edge};

pub(crate) fn validate_workspace_source_reachability(
    root: &Path,
    relative_workspace: &str,
) -> Result<Vec<Diagnostic>, String> {
    analysis::enforce_workspace_source_reachability(root, relative_workspace)
}

/// Run all source-level constitutional rules over configured subworkspace crates.
pub(crate) fn validate_source_rules(
    root: &Path,
    subworkspaces: &[SubworkspaceConfig],
    naming: &NamingConfig,
    law_substrates: &[LawSubstrateConfig],
    query_audience: &QueryAudienceContract,
    facade_exports: &FacadeVocabularyAuthority<'_>,
) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(law_substrates::validate_law_substrates(
        naming,
        law_substrates,
    ));

    diagnostics.extend(analysis::validate(
        root,
        subworkspaces,
        query_audience,
        facade_exports,
    )?);

    Ok(diagnostics)
}
