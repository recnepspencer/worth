use std::path::Path;

use crate::{diagnostics, paths, session, CompilerFixtureRefusal};

#[derive(Clone, Copy)]
pub(crate) enum RequiredEnvironment {
    Any,
    Production,
}

/// Check a real prebuilt denial against its complete `.stderr` snapshot.
pub fn denied(path: impl AsRef<Path>) -> Result<(), CompilerFixtureRefusal> {
    denied_paths(path.as_ref(), RequiredEnvironment::Any)
}

/// Additionally require that the fixture was compiled in the production graph.
pub fn production_denial(path: impl AsRef<Path>) -> Result<(), CompilerFixtureRefusal> {
    denied_paths(path.as_ref(), RequiredEnvironment::Production)
}

fn denied_paths(
    pattern: &Path,
    required: RequiredEnvironment,
) -> Result<(), CompilerFixtureRefusal> {
    let outcomes = session::outcomes()?;
    let selected = paths::expand(pattern)?;
    for path in selected {
        let canonical = path
            .canonicalize()
            .map_err(|error| CompilerFixtureRefusal::Read {
                path: path.clone(),
                reason: error.to_string(),
            })?;
        let expected = paths::read(&path.with_extension("stderr"))?;
        check_outcome(
            &canonical,
            session::fixture(outcomes, &canonical)?,
            &expected,
            required,
        )?;
    }
    Ok(())
}

pub(crate) fn check_outcome(
    source: &Path,
    outcome: &serde_json::Value,
    expected: &str,
    required: RequiredEnvironment,
) -> Result<(), CompilerFixtureRefusal> {
    let environment = outcome["environment"].as_str().ok_or_else(|| {
        CompilerFixtureRefusal::InvalidPrebuild("missing compiler environment".into())
    })?;
    match environment {
        "production" | "certification" => (),
        other => {
            return Err(CompilerFixtureRefusal::InvalidPrebuild(format!(
                "unknown compiler environment {other:?}"
            )))
        }
    }
    if matches!(required, RequiredEnvironment::Production) && environment != "production" {
        return Err(CompilerFixtureRefusal::WrongEnvironment {
            path: source.to_path_buf(),
            observed: environment.to_owned(),
        });
    }
    match outcome["fate"].as_str() {
        Some("denied") => (),
        Some("compiled") => {
            return Err(CompilerFixtureRefusal::UnexpectedCompilerSuccess(
                source.to_path_buf(),
            ))
        }
        other => {
            return Err(CompilerFixtureRefusal::InvalidPrebuild(format!(
                "unknown fixture fate {other:?}"
            )))
        }
    }
    let messages = outcome["messages"].as_array().ok_or_else(|| {
        CompilerFixtureRefusal::InvalidPrebuild("missing raw compiler diagnostics".into())
    })?;
    let mut observed = String::new();
    let mut attributed = false;
    let mut error_count = 0;
    for (index, message) in messages.iter().enumerate() {
        if message["level"] == "failure-note" {
            continue;
        }
        let rendered = message["rendered"]
            .as_str()
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| CompilerFixtureRefusal::MissingRenderedDiagnostic {
                path: source.to_path_buf(),
                index,
            })?;
        observed.push_str(rendered);
        if message["level"] == "error" {
            attributed |= diagnostics::attributed(message, source);
            error_count += 1;
        }
    }
    if error_count == 0 {
        return Err(CompilerFixtureRefusal::NoCompilerErrors(
            source.to_path_buf(),
        ));
    }
    if !attributed {
        return Err(CompilerFixtureRefusal::UnattributedDiagnostics(
            source.to_path_buf(),
        ));
    }
    let (expected, observed) = crate::snapshot::compared(expected, &observed, source)
        .map_err(CompilerFixtureRefusal::InvalidPrebuild)?;
    if expected != observed {
        return Err(CompilerFixtureRefusal::WrongDiagnostics {
            path: source.to_path_buf(),
            expected,
            observed,
        });
    }
    Ok(())
}
