//! Explicit review and recording of actual, attributed compiler denials.
use crate::{denial, paths, session, snapshot, CompilerFixtureRefusal};
use std::path::{Path, PathBuf};

/// Complete comparison under the same existing masks used by the court.
pub struct CompilerSnapshotReview {
    pub source: PathBuf,
    pub expected: String,
    pub observed: String,
}

/// Read the runner's actual receipt; never record a success, missing source,
/// unattributed error or unknown location as a protected compiler boundary.
pub fn review_denial_snapshot(
    path: impl AsRef<Path>,
) -> Result<CompilerSnapshotReview, CompilerFixtureRefusal> {
    let path = path.as_ref();
    let source = path
        .canonicalize()
        .map_err(|error| CompilerFixtureRefusal::Read {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    let outcomes = session::outcomes()?;
    let observed = denial::rendered_denial(
        &source,
        session::fixture(outcomes, &source)?,
        denial::RequiredEnvironment::Any,
    )?;
    let expected = paths::read(&source.with_extension("stderr"))?;
    let (expected, observed) = snapshot::compared(&expected, &observed, &source)
        .map_err(CompilerFixtureRefusal::InvalidPrebuild)?;
    Ok(CompilerSnapshotReview {
        source,
        expected,
        observed,
    })
}

/// Write one explicitly reviewed fixture's complete current diagnostic.
/// This does not decide that a changed diagnostic preserves its property.
pub fn record_denial_snapshot(path: impl AsRef<Path>) -> Result<(), CompilerFixtureRefusal> {
    let review = review_denial_snapshot(path)?;
    let destination = review.source.with_extension("stderr");
    write_error_snapshot(&destination, &review.observed)
}

fn write_error_snapshot(destination: &Path, observed: &str) -> Result<(), CompilerFixtureRefusal> {
    if observed
        .lines()
        .any(|line| line.starts_with("warning:") || line.starts_with("warning["))
    {
        return Err(CompilerFixtureRefusal::SnapshotWarning(
            destination.to_path_buf(),
        ));
    }
    std::fs::write(destination, format!("{observed}\n")).map_err(|error| {
        CompilerFixtureRefusal::Write {
            path: destination.to_path_buf(),
            reason: error.to_string(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_recording_refuses_lint_warnings_before_changing_the_golden() {
        let path =
            std::env::temp_dir().join(format!("worth-snapshot-warning-{}", std::process::id()));
        std::fs::write(&path, "original golden").unwrap();
        for diagnostic in [
            "error[E0451]: field is private\nwarning: function is never used",
            "warning[unused]: unrelated lint\nerror[E0624]: constructor is private",
        ] {
            assert!(matches!(
                write_error_snapshot(&path, diagnostic),
                Err(CompilerFixtureRefusal::SnapshotWarning(_))
            ));
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "original golden");
        }
        let denial =
            "error[E0451]: field is private\n  | let text = \"warning: not a diagnostic\";";
        write_error_snapshot(&path, denial).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            format!("{denial}\n")
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn missing_compiler_receipts_name_the_supported_qualification_paths() {
        let message = CompilerFixtureRefusal::MissingPrebuild.to_string();
        assert!(message.contains("python3 scripts/ci/query_gate.py root-landing"));
        assert!(message.contains("query-landing"));
    }
}
