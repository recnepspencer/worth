pub(crate) mod bounded_process;
mod cargo_environment;
mod declaration;
mod diagnostics;
mod environment_manifest;
mod result;
#[cfg(test)]
mod tests;

use std::path::Path;

pub use declaration::{
    ExpectedCompilerDenial, UiFixtureDeclaration, UiFixtureIdentity, UiProofEnvironment,
    UiProofSuiteDeclaration,
};
pub use diagnostics::CheckedCompilerDiagnostic;
pub use result::{UiFixtureResult, UiRunFailure, UiRunResult};

/// Executes one declared compiler-boundary suite in its canonical cache-sharing
/// Cargo environment. Success means every fixture failed for its declared
/// semantic reason.
pub fn run_ui_proof_suite(
    workspace_root: &Path,
    declaration: &UiProofSuiteDeclaration,
) -> Result<UiRunResult, UiRunFailure> {
    cargo_environment::run(workspace_root, declaration)
}

/// Concise declaration path for the common case where one source directory and
/// one dependency environment own a set of semantic compile denials.
pub fn run_cargo_ui_fixture_suite(
    workspace_root: &Path,
    suite_identity: &str,
    dependency_manifest: String,
    feature_identity: &str,
    profile_identity: &str,
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
    let environment =
        UiProofEnvironment::cargo(dependency_manifest, feature_identity, profile_identity)
            .map_err(UiRunFailure::InvalidDeclaration)?;
    let declaration = UiProofSuiteDeclaration::new(suite_identity, environment, fixtures)
        .map_err(UiRunFailure::InvalidDeclaration)?;
    run_ui_proof_suite(workspace_root, &declaration)
}

pub fn cargo_dependency_manifest(
    path_dependencies: &[(&str, &Path, &[&str])],
    registry_dependencies: &[(&str, &str)],
) -> String {
    let mut manifest = String::from("[dependencies]\n");
    for (name, path, features) in path_dependencies {
        let path = path.display().to_string().replace('\\', "/");
        if features.is_empty() {
            manifest.push_str(&format!("{name} = {{ path = \"{path}\" }}\n"));
        } else {
            let features = features
                .iter()
                .map(|feature| format!("\"{feature}\""))
                .collect::<Vec<_>>()
                .join(", ");
            manifest.push_str(&format!(
                "{name} = {{ path = \"{path}\", features = [{features}] }}\n"
            ));
        }
    }
    for (name, version) in registry_dependencies {
        manifest.push_str(&format!("{name} = \"{version}\"\n"));
    }
    manifest
}
