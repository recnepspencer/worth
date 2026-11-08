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
    std::fs::write(&destination, format!("{}\n", review.observed)).map_err(|error| {
        CompilerFixtureRefusal::Write {
            path: destination,
            reason: error.to_string(),
        }
    })
}
