//! Production compiler fences with semantic contracts for selected fixtures.
use std::path::{Component, Path};

use crate::denial::{check_outcome, RequiredEnvironment};
use crate::semantic_denial::{
    matches_error, semantic_errors, validate_requirement, RequiredDiagnostic,
};
use crate::{paths, session, CompilerFixtureRefusal};

mod assignment;

/// One error obligation in a production/check compiler observation.
#[derive(Clone, Copy)]
pub enum ProductionDiagnosticExpectation<'a> {
    Code {
        code: &'a str,
        symbols: &'a [&'a str],
    },
    Uncoded {
        cause: &'a str,
        symbols: &'a [&'a str],
    },
}

impl<'a> ProductionDiagnosticExpectation<'a> {
    fn required(self) -> (RequiredDiagnostic<'a>, &'a [&'a str]) {
        match self {
            Self::Code { code, symbols } => (RequiredDiagnostic::Code(code), symbols),
            Self::Uncoded { cause, symbols } => {
                (RequiredDiagnostic::ExplicitNullCause(cause), symbols)
            }
        }
    }
}

/// All distinct error obligations for one exact selected fixture basename.
pub struct ProductionSemanticDenialContract<'a> {
    pub fixture: &'a str,
    pub errors: &'a [ProductionDiagnosticExpectation<'a>],
}

/// Check every selected fixture in the actual production/check graph.
/// Contracted fixtures require exactly their declared distinct errors; all
/// other selected fixtures retain their complete rendered snapshot contract.
pub fn production_denials(
    pattern: impl AsRef<Path>,
    contracts: &[ProductionSemanticDenialContract<'_>],
) -> Result<(), CompilerFixtureRefusal> {
    let pattern = pattern.as_ref();
    let selected = paths::expand(pattern)?;
    let resolved = resolve_contracts(pattern, &selected, contracts)?;
    let outcomes = session::outcomes()?;
    let mut refusals = Vec::new();
    for (path, contract) in selected.iter().zip(resolved) {
        let result = (|| {
            let source = path
                .canonicalize()
                .map_err(|error| CompilerFixtureRefusal::Read {
                    path: path.clone(),
                    reason: error.to_string(),
                })?;
            let outcome = session::fixture(outcomes, &source)?;
            if outcome["proof_stage"] != "check" {
                return Err(CompilerFixtureRefusal::InvalidPrebuild(
                    "production denial batch requires check evidence".into(),
                ));
            }
            match contract {
                Some(contract) => check_production_outcome(&source, outcome, contract.errors),
                None => check_outcome(
                    &source,
                    outcome,
                    &paths::read(&path.with_extension("stderr"))?,
                    RequiredEnvironment::Production,
                ),
            }
        })();
        if let Err(refusal) = result {
            refusals.push(refusal);
        }
    }
    match refusals.len() {
        0 => Ok(()),
        1 => Err(refusals.pop().expect("one collected refusal")),
        _ => Err(CompilerFixtureRefusal::Multiple(refusals)),
    }
}

fn resolve_contracts<'a>(
    pattern: &Path,
    selected: &[std::path::PathBuf],
    contracts: &'a [ProductionSemanticDenialContract<'_>],
) -> Result<Vec<Option<&'a ProductionSemanticDenialContract<'a>>>, CompilerFixtureRefusal> {
    let mut result = vec![None; selected.len()];
    for contract in contracts {
        let mut components = Path::new(contract.fixture).components();
        if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
            return Err(CompilerFixtureRefusal::InvalidPrebuild(
                "semantic contract requires one exact fixture basename".into(),
            ));
        }
        let expected = paths::source_key(&pattern.with_file_name(contract.fixture));
        let indices = selected
            .iter()
            .enumerate()
            .filter(|(_, path)| paths::source_key(path) == expected)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let [index] = indices.as_slice() else {
            return Err(CompilerFixtureRefusal::InvalidPrebuild(format!(
                "semantic fixture {} is not selected exactly once",
                contract.fixture
            )));
        };
        if result[*index].replace(contract).is_some() {
            return Err(CompilerFixtureRefusal::InvalidPrebuild(format!(
                "duplicate semantic fixture contract {}",
                contract.fixture
            )));
        }
    }
    Ok(result)
}

fn check_production_outcome(
    source: &Path,
    outcome: &serde_json::Value,
    requirements: &[ProductionDiagnosticExpectation<'_>],
) -> Result<(), CompilerFixtureRefusal> {
    if requirements.is_empty() {
        return Err(CompilerFixtureRefusal::InvalidPrebuild(
            "production semantic contract requires error obligations".into(),
        ));
    }
    for requirement in requirements {
        let (required, symbols) = requirement.required();
        validate_requirement(required, symbols)?;
    }
    let errors = semantic_errors(source, outcome, RequiredEnvironment::Production)?;
    let candidates = requirements
        .iter()
        .map(|requirement| {
            let (required, symbols) = requirement.required();
            errors
                .iter()
                .enumerate()
                .filter(|(_, message)| matches_error(message, source, required, symbols))
                .map(|(index, _)| index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if !assignment::covers_all(&candidates, errors.len()) {
        return Err(CompilerFixtureRefusal::WrongSemanticDenial {
            path: source.to_path_buf(),
            reason: format!(
                "{} distinct declared errors do not match the {} actual production errors",
                requirements.len(),
                errors.len()
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
