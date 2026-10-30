use std::path::PathBuf;

/// Why a court could not establish its declared compiler boundary.
#[derive(Debug)]
pub enum CompilerFixtureRefusal {
    MissingPrebuild,
    Read {
        path: PathBuf,
        reason: String,
    },
    InvalidPrebuild(String),
    ChangedPrebuild {
        original: PathBuf,
        observed: PathBuf,
    },
    UnobservedFixture(PathBuf),
    UnexpectedCompilerSuccess(PathBuf),
    WrongEnvironment {
        path: PathBuf,
        observed: String,
    },
    NoCompilerErrors(PathBuf),
    UnattributedDiagnostics(PathBuf),
    MissingRenderedDiagnostic {
        path: PathBuf,
        index: usize,
    },
    WrongDiagnostics {
        path: PathBuf,
        expected: String,
        observed: String,
    },
    EmptyPattern(PathBuf),
    UnsupportedPattern(PathBuf),
}

impl std::fmt::Display for CompilerFixtureRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPrebuild => write!(formatter, "missing WORTH_PREBUILT_COMPILER_OUTCOMES; run the workspace gate prebuild"),
            Self::Read { path, reason } => write!(formatter, "read {}: {reason}", path.display()),
            Self::InvalidPrebuild(reason) => write!(formatter, "invalid compiler prebuild: {reason}"),
            Self::ChangedPrebuild { original, observed } => write!(formatter, "compiler prebuild changed during one court process: {} -> {}", original.display(), observed.display()),
            Self::UnobservedFixture(path) => write!(formatter, "compiler did not observe {}", path.display()),
            Self::UnexpectedCompilerSuccess(path) => write!(formatter, "{} unexpectedly compiled", path.display()),
            Self::WrongEnvironment { path, observed } => write!(formatter, "{} requires a production compiler graph, observed {observed}", path.display()),
            Self::NoCompilerErrors(path) => write!(formatter, "{} has no raw compiler errors", path.display()),
            Self::UnattributedDiagnostics(path) => write!(formatter, "raw compiler errors contain no span or expansion callsite for {}", path.display()),
            Self::MissingRenderedDiagnostic { path, index } => write!(formatter, "{} error {index} omits its rendered diagnostic", path.display()),
            Self::WrongDiagnostics { path, expected, observed } => write!(formatter, "{} differs from its compiler snapshot\nexpected:\n{expected}\nobserved:\n{observed}", path.display()),
            Self::EmptyPattern(path) => write!(formatter, "compiler fixture pattern {} selects no source", path.display()),
            Self::UnsupportedPattern(path) => write!(formatter, "compiler fixture pattern {} requires an explicit supported expansion", path.display()),
        }
    }
}

impl std::error::Error for CompilerFixtureRefusal {}
