use std::path::PathBuf;

use super::{CheckedCompilerDiagnostic, UiCompilerEnvironment, UiFixtureIdentity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRunResult {
    pub environment_identity: String,
    pub shared_target_root: String,
    pub fixtures: Vec<UiFixtureResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiFixtureResult {
    pub fixture: UiFixtureIdentity,
    pub diagnostics: Vec<CheckedCompilerDiagnostic>,
    pub semantic_denial_matched: bool,
}

#[derive(Debug)]
pub enum UiRunFailure {
    InvalidDeclaration(String),
    MissingPrebuiltOutcomes {
        path: Option<PathBuf>,
        reason: String,
    },
    MalformedPrebuiltOutcomes {
        path: PathBuf,
        reason: String,
    },
    FixtureRead(String),
    UnobservedFixture(String),
    WrongCompilerEnvironment {
        fixture: String,
        expected: UiCompilerEnvironment,
        observed: UiCompilerEnvironment,
    },
    UnexpectedCompilerSuccess(String),
    WrongCompilerDenial {
        fixture: String,
        reason: String,
        diagnostics: Vec<CheckedCompilerDiagnostic>,
    },
}

impl std::fmt::Display for UiRunFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDeclaration(reason) => write!(formatter, "invalid UI declaration: {reason}"),
            Self::MissingPrebuiltOutcomes { path, reason } => {
                write!(formatter, "prebuilt UI outcomes are missing at {path:?}: {reason}; run the workspace prebuild step")
            }
            Self::MalformedPrebuiltOutcomes { path, reason } => {
                write!(formatter, "invalid prebuilt UI outcomes {}: {reason}", path.display())
            }
            Self::FixtureRead(reason) => write!(formatter, "could not read UI fixture: {reason}"),
            Self::UnobservedFixture(fixture) => {
                write!(formatter, "UI fixture {fixture} has no observed compiler outcome; run the workspace prebuild step")
            }
            Self::WrongCompilerEnvironment { fixture, expected, observed } => {
                write!(formatter, "UI fixture {fixture} requires {} compiler outcomes, found {}", expected.as_str(), observed.as_str())
            }
            Self::UnexpectedCompilerSuccess(fixture) => {
                write!(formatter, "UI fixture {fixture} unexpectedly compiled")
            }
            Self::WrongCompilerDenial {
                fixture,
                reason,
                diagnostics,
            } => write!(
                formatter,
                "UI fixture {fixture} failed for the wrong reason: {reason}; diagnostics={diagnostics:?}"
            ),
        }
    }
}

impl std::error::Error for UiRunFailure {}
