use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::diagnostics::{checked_diagnostics, validate_denial};
use super::{
    bounded_process, environment_manifest, UiFixtureIdentity, UiFixtureResult,
    UiProofSuiteDeclaration, UiRunFailure, UiRunResult,
};

const COMPILER_TIMEOUT: Duration = Duration::from_secs(300);
const OUTPUT_LIMIT: usize = 16 * 1024 * 1024;

pub(super) fn run(
    workspace_root: &Path,
    declaration: &UiProofSuiteDeclaration,
) -> Result<UiRunResult, UiRunFailure> {
    let workspace_root =
        cargo_compatible_path(&workspace_root.canonicalize().map_err(|error| {
            UiRunFailure::EnvironmentObservation(format!(
                "canonicalize {}: {error}",
                workspace_root.display()
            ))
        })?);
    let manifest_contract = environment_manifest::UiEnvironmentManifestContract::load(
        &workspace_root,
        declaration.environment().profile_identity(),
    )?;
    let environment_identity = digest_serialized(&(
        declaration.suite_identity(),
        declaration.environment().dependency_manifest(),
        declaration.environment().feature_identity(),
        declaration.environment().profile_identity(),
    ))?;
    let environment_root = workspace_root
        .join("target/store-ui/environments")
        .join(&environment_identity[..24]);
    let source_root = environment_root.join("src/bin");
    if source_root.exists() {
        fs::remove_dir_all(&source_root)
            .map_err(|error| UiRunFailure::EnvironmentObservation(error.to_string()))?;
    }
    fs::create_dir_all(&source_root)
        .map_err(|error| UiRunFailure::EnvironmentObservation(error.to_string()))?;
    let manifest_path = environment_root.join("Cargo.toml");
    let manifest = manifest_contract.render(
        &environment_identity,
        declaration.environment().dependency_manifest(),
    )?;
    fs::write(&manifest_path, manifest)
        .map_err(|error| UiRunFailure::EnvironmentObservation(error.to_string()))?;

    let target_root = workspace_root.join("target/store-ui/cargo-target");
    fs::create_dir_all(&target_root)
        .map_err(|error| UiRunFailure::EnvironmentObservation(error.to_string()))?;
    let mut fixtures = Vec::with_capacity(declaration.fixtures().len());
    let environment = FixtureEnvironment {
        workspace_root: &workspace_root,
        source_root: &source_root,
        manifest_path: &manifest_path,
        target_root: &target_root,
        environment_identity: &environment_identity,
        declaration,
    };
    for fixture in declaration.fixtures() {
        fixtures.push(run_fixture(&environment, fixture)?);
    }

    Ok(UiRunResult {
        environment_identity,
        shared_target_root: normalized_path(&workspace_root, &target_root),
        fixtures,
    })
}

struct FixtureEnvironment<'a> {
    workspace_root: &'a Path,
    source_root: &'a Path,
    manifest_path: &'a Path,
    target_root: &'a Path,
    environment_identity: &'a str,
    declaration: &'a UiProofSuiteDeclaration,
}

fn run_fixture(
    environment: &FixtureEnvironment<'_>,
    fixture: &super::UiFixtureDeclaration,
) -> Result<UiFixtureResult, UiRunFailure> {
    let source = fs::read(fixture.source_path()).map_err(|error| {
        UiRunFailure::FixtureRead(format!("{}: {error}", fixture.source_path().display()))
    })?;
    let identity = UiFixtureIdentity {
        suite_identity: environment.declaration.suite_identity().to_owned(),
        case_identity: fixture.case_identity().to_owned(),
        source_path: normalized_path(environment.workspace_root, fixture.source_path()),
        environment_identity: environment.environment_identity.to_owned(),
        expected_denial_identity: digest_serialized(fixture.expected_denial())?,
    };
    let bin_name = fixture_bin_name(fixture.case_identity());
    fs::write(
        environment.source_root.join(format!("{bin_name}.rs")),
        source,
    )
    .map_err(|error| UiRunFailure::EnvironmentObservation(error.to_string()))?;

    let mut command = Command::new("cargo");
    command
        .args([
            "check",
            "--offline",
            "--message-format=json",
            "--profile",
            environment.declaration.environment().profile_identity(),
            "--bin",
            &bin_name,
            "--manifest-path",
        ])
        .arg(environment.manifest_path)
        .env("CARGO_TARGET_DIR", environment.target_root)
        .current_dir(environment.workspace_root);
    let output = bounded_process::run(&mut command, COMPILER_TIMEOUT, OUTPUT_LIMIT)
        .map_err(UiRunFailure::CompilerLaunch)?;
    if output.timed_out {
        return Err(UiRunFailure::CompilerTimedOut(
            fixture.case_identity().to_owned(),
        ));
    }
    if output.status.success() {
        return Err(UiRunFailure::UnexpectedCompilerSuccess(
            fixture.case_identity().to_owned(),
        ));
    }

    let root = environment.workspace_root.to_string_lossy();
    let diagnostics = checked_diagnostics(&output.stdout, &root);
    let stderr = String::from_utf8_lossy(&output.stderr)
        .replace(root.as_ref(), "<workspace>")
        .replace('\\', "/");
    validate_denial(fixture.expected_denial(), &diagnostics, &stderr).map_err(|reason| {
        UiRunFailure::WrongCompilerDenial {
            fixture: fixture.case_identity().to_owned(),
            reason,
            diagnostics: diagnostics.clone(),
        }
    })?;
    Ok(UiFixtureResult {
        fixture: identity,
        diagnostics,
        semantic_denial_matched: true,
    })
}

fn fixture_bin_name(case: &str) -> String {
    case.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned()
}

fn normalized_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(windows)]
fn cargo_compatible_path(path: &Path) -> std::path::PathBuf {
    let value = path.to_string_lossy();
    if let Some(unc) = value.strip_prefix(r"\\?\UNC\") {
        return std::path::PathBuf::from(format!(r"\\{unc}"));
    }
    value
        .strip_prefix(r"\\?\")
        .map_or_else(|| path.to_path_buf(), std::path::PathBuf::from)
}

#[cfg(not(windows))]
fn cargo_compatible_path(path: &Path) -> std::path::PathBuf {
    path.to_path_buf()
}

fn digest_serialized(value: &impl Serialize) -> Result<String, UiRunFailure> {
    serde_json::to_vec(value)
        .map(|bytes| digest_bytes(&bytes))
        .map_err(|error| UiRunFailure::InvalidDeclaration(error.to_string()))
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
