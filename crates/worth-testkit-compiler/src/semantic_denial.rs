use std::path::Path;

use crate::denial::{denied_messages, RequiredEnvironment};
use crate::{diagnostics, session, CompilerFixtureRefusal};

/// Require a certification/check denial whose code and API symbols belong to
/// one raw compiler error attributed to this exact fixture, without a prose golden.
pub fn certification_semantic_denial(
    path: impl AsRef<Path>,
    code: &str,
    symbols: &[&str],
) -> Result<(), CompilerFixtureRefusal> {
    check_path(path.as_ref(), RequiredDiagnostic::Code(code), symbols)
}

/// Require an explicitly uncoded certification/check error whose semantic cause
/// and API symbols belong to this exact fixture. Missing code fields do not match.
pub fn certification_uncoded_semantic_denial(
    path: impl AsRef<Path>,
    cause: &str,
    symbols: &[&str],
) -> Result<(), CompilerFixtureRefusal> {
    check_path(
        path.as_ref(),
        RequiredDiagnostic::ExplicitNullCause(cause),
        symbols,
    )
}

#[derive(Clone, Copy)]
pub(crate) enum RequiredDiagnostic<'a> {
    Code(&'a str),
    ExplicitNullCause(&'a str),
}

fn check_path(
    path: &Path,
    required: RequiredDiagnostic<'_>,
    symbols: &[&str],
) -> Result<(), CompilerFixtureRefusal> {
    let outcomes = session::outcomes()?;
    let source = path
        .canonicalize()
        .map_err(|error| CompilerFixtureRefusal::Read {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    check_semantic_outcome(
        &source,
        session::fixture(outcomes, &source)?,
        required,
        symbols,
    )
}

pub(crate) fn check_semantic_outcome(
    source: &Path,
    outcome: &serde_json::Value,
    required: RequiredDiagnostic<'_>,
    symbols: &[&str],
) -> Result<(), CompilerFixtureRefusal> {
    let fragment = match required {
        RequiredDiagnostic::Code(code) => code,
        RequiredDiagnostic::ExplicitNullCause(cause) => cause,
    };
    if fragment.trim().is_empty()
        || symbols.is_empty()
        || symbols.iter().any(|symbol| symbol.trim().is_empty())
    {
        return Err(CompilerFixtureRefusal::InvalidPrebuild(
            match required {
                RequiredDiagnostic::Code(_) => {
                    "semantic denial requires a code and nonempty API symbols"
                }
                RequiredDiagnostic::ExplicitNullCause(_) => {
                    "uncoded semantic denial requires a cause and nonempty API symbols"
                }
            }
            .into(),
        ));
    }
    let messages = denied_messages(source, outcome, RequiredEnvironment::Any)?;
    validate_semantic_observation(outcome, messages)?;
    let mut matched = false;
    let mut errors = 0;
    for message in messages
        .iter()
        .filter(|message| message["level"] == "error")
    {
        errors += 1;
        let semantic = diagnostics::semantic_text(message);
        let rendered = message["rendered"].as_str().unwrap_or_default();
        if SETUP_FAILURES
            .iter()
            .any(|fragment| semantic.contains(fragment) || rendered.contains(fragment))
        {
            return Err(CompilerFixtureRefusal::WrongSemanticDenial {
                path: source.to_path_buf(),
                reason: "a compiler error reports a setup failure".into(),
            });
        }
        let expected = match required {
            RequiredDiagnostic::Code(code) => message["code"]["code"] == code,
            RequiredDiagnostic::ExplicitNullCause(cause) => {
                message.get("code").is_some_and(serde_json::Value::is_null)
                    && semantic.contains(cause)
            }
        };
        matched |= expected
            && symbols.iter().all(|symbol| semantic.contains(symbol))
            && diagnostics::attributed(message, source);
    }
    if errors == 0 {
        return Err(CompilerFixtureRefusal::NoCompilerErrors(
            source.to_path_buf(),
        ));
    }
    if !matched {
        return Err(CompilerFixtureRefusal::WrongSemanticDenial {
            path: source.to_path_buf(),
            reason: match required {
                RequiredDiagnostic::Code(code) => format!("no fixture-attributed error has code {code} and symbols {symbols:?}"),
                RequiredDiagnostic::ExplicitNullCause(cause) => format!("no fixture-attributed error has explicit null code, cause {cause:?} and symbols {symbols:?}"),
            },
        });
    }
    Ok(())
}

fn validate_semantic_observation(
    outcome: &serde_json::Value,
    messages: &[serde_json::Value],
) -> Result<(), CompilerFixtureRefusal> {
    let fields = [
        "environment",
        "environment_identity",
        "audience_identity",
        "shared_target_root",
        "proof_stage",
        "fate",
        "messages",
    ];
    if !outcome.is_object()
        || outcome
            .as_object()
            .unwrap()
            .keys()
            .any(|key| !fields.contains(&key.as_str()))
    {
        return Err(CompilerFixtureRefusal::InvalidPrebuild(
            "unknown semantic observation field".into(),
        ));
    }
    if outcome["environment"] != "certification" || outcome["proof_stage"] != "check" {
        return Err(CompilerFixtureRefusal::InvalidPrebuild(
            "semantic denial requires certification/check evidence".into(),
        ));
    }
    for message in messages {
        validate_message(message).map_err(CompilerFixtureRefusal::InvalidPrebuild)?;
    }
    Ok(())
}

fn validate_message(message: &serde_json::Value) -> Result<(), String> {
    if !message.is_object()
        || !message["message"].is_string()
        || !message["level"].is_string()
        || !(message["code"].is_null() || message["code"]["code"].is_string())
        || !(message["rendered"].is_null() || message["rendered"].is_string())
    {
        return Err("malformed raw semantic diagnostic".into());
    }
    let spans = message["spans"]
        .as_array()
        .ok_or("raw semantic diagnostic omits spans")?;
    let children = message["children"]
        .as_array()
        .ok_or("raw semantic diagnostic omits children")?;
    for span in spans {
        validate_span(span)?;
    }
    for child in children {
        validate_message(child)?;
    }
    Ok(())
}

fn validate_span(span: &serde_json::Value) -> Result<(), String> {
    if !span.is_object()
        || !span["file_name"].is_string()
        || !(span["label"].is_null() || span["label"].is_string())
    {
        return Err("malformed raw semantic span".into());
    }
    if let Some(expansion) = span.get("expansion").filter(|value| !value.is_null()) {
        validate_span(&expansion["span"])?;
    }
    Ok(())
}

const SETUP_FAILURES: [&str; 8] = [
    "failed to load manifest",
    "no matching package",
    "can't find crate",
    "failed to get `",
    "failed to load source for dependency",
    "couldn't read",
    "No such file or directory",
    "does not have these features",
];
