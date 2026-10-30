use std::path::Path;

use super::{
    prebuilt_environment, ExpectedCompilerDenial, UiCompilerEnvironment, UiFixtureDeclaration,
    UiProofSuiteDeclaration, UiRunFailure, UiRunResult,
};

/// Reads actual prebuilt compiler outcomes and checks every declared denial.
/// The workspace runner owns compilation, dependency graphs and profiles.
pub fn read_ui_proof_suite(
    workspace_root: &Path,
    declaration: &UiProofSuiteDeclaration,
) -> Result<UiRunResult, UiRunFailure> {
    prebuilt_environment::read(workspace_root, declaration)
}

/// Reads one fixture family in its explicit production or certification graph.
pub fn read_ui_fixture_suite(
    workspace_root: &Path,
    suite_identity: &str,
    environment: UiCompilerEnvironment,
    source_root: &Path,
    fixtures: &[(&str, &[&str])],
) -> Result<UiRunResult, UiRunFailure> {
    let fixtures = fixtures
        .iter()
        .map(|(name, fragments)| {
            UiFixtureDeclaration::new(
                name.trim_end_matches(".rs"),
                source_root.join(name),
                ExpectedCompilerDenial::semantic_fragments(fragments.iter().copied())
                    .map_err(UiRunFailure::InvalidDeclaration)?,
            )
            .map_err(UiRunFailure::InvalidDeclaration)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let declaration = UiProofSuiteDeclaration::new(suite_identity, environment, fixtures)
        .map_err(UiRunFailure::InvalidDeclaration)?;
    read_ui_proof_suite(workspace_root, &declaration)
}
