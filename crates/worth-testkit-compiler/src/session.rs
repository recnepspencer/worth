use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::{paths, CompilerFixtureRefusal};

struct CapturedOutcomes {
    path: PathBuf,
    document: serde_json::Value,
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
    Ok(document)
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
