//! Positive compiler counterparts consume the same attributed outer receipts.
use std::path::Path;

use crate::{session, CompilerFixtureRefusal};

/// Require successful compilation of this exact source in the outer session.
/// No absent, denied or malformed observation can stand for a positive twin.
pub fn compiled(path: impl AsRef<Path>) -> Result<(), CompilerFixtureRefusal> {
    let outcomes = session::outcomes()?;
    let source = path
        .as_ref()
        .canonicalize()
        .map_err(|error| CompilerFixtureRefusal::Read {
            path: path.as_ref().to_path_buf(),
            reason: error.to_string(),
        })?;
    let outcome = session::fixture(outcomes, &source)?;
    check_compiled(&source, outcome)
}

fn check_compiled(
    source: &Path,
    outcome: &serde_json::Value,
) -> Result<(), CompilerFixtureRefusal> {
    session::validate_provenance(outcome)?;
    if outcome["fate"] != "compiled"
        || !matches!(
            outcome["environment"].as_str(),
            Some("production" | "certification")
        )
        || !matches!(outcome["proof_stage"].as_str(), Some("check" | "codegen"))
        || !outcome["messages"].as_array().is_some_and(|messages| {
            messages.iter().all(|message| {
                message.is_object()
                    && message["level"]
                        .as_str()
                        .is_some_and(|level| level != "error")
            })
        })
    {
        return Err(CompilerFixtureRefusal::InvalidPrebuild(format!(
            "{} has no successful compiler receipt",
            source.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn positive_counterpart_refuses_denied_missing_and_malformed_receipts() {
        let source = Path::new("consumer.rs");
        let observed = json!({"environment":"certification", "environment_identity":"batch",
            "audience_identity":"consumer", "shared_target_root":"target",
            "proof_stage":"check", "fate":"compiled", "messages":[]});
        assert!(check_compiled(source, &observed).is_ok());
        for (field, value) in [
            ("fate", "denied"),
            ("fate", "unobserved"),
            ("environment", "unknown"),
            ("proof_stage", "unknown"),
            ("audience_identity", ""),
        ] {
            let mut changed = observed.clone();
            changed[field] = value.into();
            assert!(check_compiled(source, &changed).is_err(), "{field}");
        }
        let mut malformed = observed.clone();
        malformed["messages"] = json!([7]);
        assert!(check_compiled(source, &malformed).is_err());
        let mut changed = observed;
        changed["messages"] = json!([{"level":"error", "message":"foreign schema"}]);
        assert!(check_compiled(source, &changed).is_err());
    }
}
