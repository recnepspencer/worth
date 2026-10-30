use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::{paths, CompilerFixtureRefusal};

struct CapturedOutcomes {
    path: PathBuf,
    document: serde_json::Value,
}

struct CompilerProvenance<'a> {
    environment_identity: &'a str,
    audience_identity: &'a str,
    shared_target_root: &'a str,
}

static OUTCOMES: OnceLock<CapturedOutcomes> = OnceLock::new();

pub(crate) fn outcomes() -> Result<&'static serde_json::Value, CompilerFixtureRefusal> {
    let path = std::env::var_os("WORTH_PREBUILT_COMPILER_OUTCOMES")
        .map(PathBuf::from)
        .ok_or(CompilerFixtureRefusal::MissingPrebuild)?;
    let captured = match OUTCOMES.get() {
        Some(captured) => captured,
        None => {
            let document = parse(&paths::read(&path)?)?;
            OUTCOMES.get_or_init(|| CapturedOutcomes {
                path: path.clone(),
                document,
            })
        }
    };
    if captured.path != path {
        return Err(CompilerFixtureRefusal::ChangedPrebuild {
            original: captured.path.clone(),
            observed: path,
        });
    }
    Ok(&captured.document)
}

pub(crate) fn parse(source: &str) -> Result<serde_json::Value, CompilerFixtureRefusal> {
    let document: serde_json::Value = serde_json::from_str(source)
        .map_err(|error| CompilerFixtureRefusal::InvalidPrebuild(error.to_string()))?;
    if document["schema"].as_u64() != Some(1) {
        return Err(CompilerFixtureRefusal::InvalidPrebuild(
            "unsupported schema".into(),
        ));
    }
    if !document["fixtures"].is_object() {
        return Err(CompilerFixtureRefusal::InvalidPrebuild(
            "missing fixture map".into(),
        ));
    }
    for outcome in document["fixtures"].as_object().unwrap().values() {
        validate_provenance(outcome)?;
    }
    Ok(document)
}

pub(crate) fn validate_provenance(
    outcome: &serde_json::Value,
) -> Result<(), CompilerFixtureRefusal> {
    let record = CompilerProvenance {
        environment_identity: required_provenance_field(outcome, "environment_identity")?,
        audience_identity: required_provenance_field(outcome, "audience_identity")?,
        shared_target_root: required_provenance_field(outcome, "shared_target_root")?,
    };
    if [
        record.environment_identity,
        record.audience_identity,
        record.shared_target_root,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
    {
        return Err(CompilerFixtureRefusal::InvalidPrebuild(
            "compiler batch, audience and target identities must be nonempty".into(),
        ));
    }
    Ok(())
}

fn required_provenance_field<'a>(
    outcome: &'a serde_json::Value,
    field: &str,
) -> Result<&'a str, CompilerFixtureRefusal> {
    outcome[field].as_str().ok_or_else(|| {
        CompilerFixtureRefusal::InvalidPrebuild(format!(
            "missing compiler provenance field {field}"
        ))
    })
}

pub(crate) fn fixture<'a>(
    document: &'a serde_json::Value,
    source: &Path,
) -> Result<&'a serde_json::Value, CompilerFixtureRefusal> {
    let key = paths::source_key(source);
    document["fixtures"]
        .get(key.as_str())
        .ok_or_else(|| CompilerFixtureRefusal::UnobservedFixture(source.to_path_buf()))
}
