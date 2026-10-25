use super::{CheckedCompilerDiagnostic, UiFixtureIdentity};

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
    EnvironmentObservation(String),
    FixtureRead(String),
    CompilerLaunch(String),
    CompilerTimedOut {
        fixture: String,
        stderr: String,
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
            Self::EnvironmentObservation(reason) => {
                write!(formatter, "could not establish UI environment: {reason}")
            }
            Self::FixtureRead(reason) => write!(formatter, "could not read UI fixture: {reason}"),
            Self::CompilerLaunch(reason) => write!(formatter, "could not launch UI compiler: {reason}"),
            Self::CompilerTimedOut { fixture, stderr } => {
                write!(formatter, "UI fixture {fixture} exceeded its compiler timeout; Cargo progress:\n{stderr}")
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
