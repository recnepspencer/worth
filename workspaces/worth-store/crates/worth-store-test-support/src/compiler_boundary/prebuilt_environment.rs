use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::diagnostics::{checked_diagnostics, validate_denial};
use super::{
    UiCompilerEnvironment, UiFixtureDeclaration, UiFixtureIdentity, UiFixtureResult,
    UiProofSuiteDeclaration, UiRunFailure, UiRunResult,
};

pub(super) fn read(
    workspace_root: &Path,
    declaration: &UiProofSuiteDeclaration,
) -> Result<UiRunResult, UiRunFailure> {
    let path = std::env::var_os("WORTH_PREBUILT_COMPILER_OUTCOMES")
        .map(PathBuf::from)
        .ok_or_else(|| UiRunFailure::MissingPrebuiltOutcomes {
            path: None,
            reason: "WORTH_PREBUILT_COMPILER_OUTCOMES is not set".to_owned(),
        })?;
    let bytes = std::fs::read(&path).map_err(|error| UiRunFailure::MissingPrebuiltOutcomes {
        path: Some(path.clone()),
        reason: error.to_string(),
    })?;
    let outcomes = decode(&bytes, &path)?;
    read_declared_suite(workspace_root, declaration, &outcomes, &path)
}

fn read_declared_suite(
    workspace_root: &Path,
    declaration: &UiProofSuiteDeclaration,
    outcomes: &PrebuiltCompilerOutcomes,
    outcomes_path: &Path,
) -> Result<UiRunResult, UiRunFailure> {
    if declaration.fixtures().is_empty() {
        return Err(UiRunFailure::InvalidDeclaration(
            "UI proof suite has no fixtures".to_owned(),
        ));
    }
    let mut result = UiRunResult {
        environment_identity: String::new(),
        shared_target_root: String::new(),
        fixtures: Vec::with_capacity(declaration.fixtures().len()),
    };
    for fixture in declaration.fixtures() {
        let source = fixture.source_path().canonicalize().map_err(|error| {
            UiRunFailure::FixtureRead(format!("{}: {error}", fixture.source_path().display()))
        })?;
        let source_key = normalized_source_key(&source);
        let observed = outcomes
            .fixtures
            .get(&source_key)
            .ok_or(UiRunFailure::UnobservedFixture(source_key))?;
        validate_observation(declaration.environment(), fixture, observed)?;
        if result.fixtures.is_empty() {
            result.environment_identity = observed.environment_identity.clone();
            result.shared_target_root = observed.shared_target_root.clone();
        } else if result.environment_identity != observed.environment_identity
            || result.shared_target_root != observed.shared_target_root
        {
            return Err(malformed(
                outcomes_path,
                format!(
                    "suite {} mixes compiler batches or target roots",
                    declaration.suite_identity()
                ),
            ));
        }
        result.fixtures.push(read_fixture(
            workspace_root,
            declaration,
            fixture,
            &source,
            observed,
            outcomes_path,
        )?);
    }
    Ok(result)
}

fn validate_observation(
    expected: UiCompilerEnvironment,
    fixture: &UiFixtureDeclaration,
    observed: &PrebuiltFixtureCompilation,
) -> Result<(), UiRunFailure> {
    if observed.environment != expected {
        return Err(UiRunFailure::WrongCompilerEnvironment {
            fixture: fixture.case_identity().to_owned(),
            expected,
            observed: observed.environment,
        });
    }
    match observed.fate {
        CompilerFate::Compiled => Err(UiRunFailure::UnexpectedCompilerSuccess(
            fixture.case_identity().to_owned(),
        )),
        CompilerFate::Denied => Ok(()),
    }
}

fn read_fixture(
    workspace_root: &Path,
    declaration: &UiProofSuiteDeclaration,
    fixture: &UiFixtureDeclaration,
    source: &Path,
    observed: &PrebuiltFixtureCompilation,
    outcomes_path: &Path,
) -> Result<UiFixtureResult, UiRunFailure> {
    for message in &observed.messages {
        validate_message_shape(message).map_err(|reason| malformed(outcomes_path, reason))?;
    }
    let diagnostics = checked_diagnostics(&observed.messages, &workspace_root.to_string_lossy());
    if diagnostics.is_empty() {
        return Err(UiRunFailure::UnobservedFixture(
            fixture.case_identity().to_owned(),
        ));
    }
    let fixture_attribution = observed
        .messages
        .iter()
        .filter(|message| message["level"] == "error")
        .map(|message| attributed_to_fixture(message, source))
        .collect::<Vec<_>>();
    validate_denial(
        fixture.expected_denial(),
        &diagnostics,
        "",
        &fixture_attribution,
    )
    .map_err(|reason| UiRunFailure::WrongCompilerDenial {
        fixture: fixture.case_identity().to_owned(),
        reason,
        diagnostics: diagnostics.clone(),
    })?;
    let expected = serde_json::to_vec(fixture.expected_denial())
        .map_err(|error| UiRunFailure::InvalidDeclaration(error.to_string()))?;
    Ok(UiFixtureResult {
        fixture: UiFixtureIdentity {
            suite_identity: declaration.suite_identity().to_owned(),
            case_identity: fixture.case_identity().to_owned(),
            source_path: fixture
                .source_path()
                .strip_prefix(workspace_root)
                .unwrap_or(fixture.source_path())
                .to_string_lossy()
                .replace('\\', "/"),
            environment_identity: observed.environment_identity.clone(),
            expected_denial_identity: format!("{:x}", Sha256::digest(&expected)),
        },
        diagnostics,
        semantic_denial_matched: true,
    })
}

fn attributed_to_fixture(message: &serde_json::Value, source: &Path) -> bool {
    message["spans"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|span| attributed_span(span, source))
        || message["children"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|child| attributed_to_fixture(child, source))
}

fn attributed_span(span: &serde_json::Value, source: &Path) -> bool {
    span["file_name"]
        .as_str()
        .is_some_and(|path| normalized_source_key(Path::new(path)) == normalized_source_key(source))
        || span
            .get("expansion")
            .filter(|expansion| !expansion.is_null())
            .is_some_and(|expansion| attributed_span(&expansion["span"], source))
}

fn decode(bytes: &[u8], path: &Path) -> Result<PrebuiltCompilerOutcomes, UiRunFailure> {
    let outcomes: PrebuiltCompilerOutcomes =
        serde_json::from_slice(bytes).map_err(|error| malformed(path, error.to_string()))?;
    if outcomes.schema != 1 {
        return Err(malformed(
            path,
            format!("unsupported schema {}", outcomes.schema),
        ));
    }
    for observed in outcomes.fixtures.values() {
        if observed.proof_stage != "check" {
            return Err(malformed(
                path,
                format!(
                    "Store compiler claims require check-stage evidence, observed {}",
                    observed.proof_stage
                ),
            ));
        }
        if observed.environment_identity.trim().is_empty()
            || observed.audience_identity.trim().is_empty()
            || observed.shared_target_root.trim().is_empty()
        {
            return Err(malformed(
                path,
                "compiler batch identity, audience identity and target root must be present",
            ));
        }
    }
    Ok(outcomes)
}

fn validate_message_shape(message: &serde_json::Value) -> Result<(), String> {
    if !message.is_object() || !message["message"].is_string() || !message["level"].is_string() {
        return Err("expected a raw rustc diagnostic with message and level".to_owned());
    }
    if !(message["code"].is_null() || message["code"]["code"].is_string())
        || !(message["rendered"].is_null() || message["rendered"].is_string())
    {
        return Err("raw rustc diagnostic has invalid code or rendered text".to_owned());
    }
    let Some(children) = message["children"].as_array() else {
        return Err("raw rustc diagnostic omits children".to_owned());
    };
    let Some(spans) = message["spans"].as_array() else {
        return Err("raw rustc diagnostic omits spans".to_owned());
    };
    for span in spans {
        if !span.is_object() || !(span["label"].is_null() || span["label"].is_string()) {
            return Err("raw rustc diagnostic has an invalid span label".to_owned());
        }
    }
    for child in children {
        validate_message_shape(child)?;
    }
    Ok(())
}

fn normalized_source_key(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    let value = if let Some(unc) = value.strip_prefix("//?/UNC/") {
        format!("//{unc}")
    } else {
        value.strip_prefix("//?/").unwrap_or(&value).to_owned()
    };
    if cfg!(windows) {
        value.to_lowercase()
    } else {
        value
    }
}

fn malformed(path: &Path, reason: impl Into<String>) -> UiRunFailure {
    UiRunFailure::MalformedPrebuiltOutcomes {
        path: path.to_path_buf(),
        reason: reason.into(),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrebuiltCompilerOutcomes {
    schema: u32,
    fixtures: BTreeMap<String, PrebuiltFixtureCompilation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrebuiltFixtureCompilation {
    environment: UiCompilerEnvironment,
    environment_identity: String,
    audience_identity: String,
    shared_target_root: String,
    proof_stage: String,
    fate: CompilerFate,
    messages: Vec<serde_json::Value>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CompilerFate {
    Denied,
    Compiled,
}

#[cfg(test)]
mod tests;
