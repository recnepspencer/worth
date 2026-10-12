//! Committed legacy-reference snapshot: parse, schema, and structural checks.
//!
//! Truth source for the *candidate* snapshot file only. Returns validated
//! candidate rows or typed structural issues - never final diagnostics.

use crate::cargo_graph::normalize_str;
use crate::legacy_references::occurrence::LegacyReferenceOccurrence;
use serde::Deserialize;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
struct LegacyReferenceSnapshotFile {
    schema_version: u32,
    #[serde(default)]
    references: Vec<SnapshotReferenceRecord>,
}

/// Wire shape of one snapshot TOML row (serde only; not shared evidence identity).
#[derive(Clone, Debug, Deserialize)]
struct SnapshotReferenceRecord {
    path: String,
    location: String,
    fragment: String,
}

/// Validated candidate baseline (schema + uniqueness already enforced).
#[derive(Debug)]
pub(super) struct ValidatedCandidateSnapshot {
    pub(super) occurrences: BTreeSet<LegacyReferenceOccurrence>,
}

/// Typed structural failure of the candidate snapshot file.
#[derive(Clone, Debug)]
pub(super) enum SnapshotIssue {
    WrongSchemaVersion {
        snapshot_relative: String,
        found: u32,
        expected: u32,
    },
    DuplicateRow {
        snapshot_relative: String,
        path: String,
        location: String,
        fragment: String,
    },
}

/// Outcome of loading the working-tree / checkout snapshot.
#[derive(Debug)]
pub(super) enum CandidateValidation {
    Issues(Vec<SnapshotIssue>),
    Valid(ValidatedCandidateSnapshot),
}

pub(super) fn load_and_validate_candidate(
    snapshot_path: &Path,
    snapshot_relative: &str,
) -> Result<CandidateValidation, String> {
    let file = load_snapshot_file(snapshot_path)?;
    Ok(validate_candidate_structure(file, snapshot_relative))
}

fn load_snapshot_file(path: &Path) -> Result<LegacyReferenceSnapshotFile, String> {
    let text = crate::source_documents::read_to_string(path)
        .map_err(|error| format!("read legacy-reference snapshot {}: {error}", path.display()))?;
    toml::from_str(&text).map_err(|error| {
        format!(
            "parse legacy-reference snapshot {}: {error}",
            path.display()
        )
    })
}

fn validate_candidate_structure(
    candidate: LegacyReferenceSnapshotFile,
    snapshot_relative: &str,
) -> CandidateValidation {
    let snapshot_relative = normalize_str(snapshot_relative);

    if candidate.schema_version != SNAPSHOT_SCHEMA_VERSION {
        return CandidateValidation::Issues(vec![SnapshotIssue::WrongSchemaVersion {
            snapshot_relative,
            found: candidate.schema_version,
            expected: SNAPSHOT_SCHEMA_VERSION,
        }]);
    }

    let mut issues = Vec::new();
    let mut seen_rows = HashSet::new();
    for row in &candidate.references {
        let key = (row.path.clone(), row.location.clone(), row.fragment.clone());
        if !seen_rows.insert(key) {
            issues.push(SnapshotIssue::DuplicateRow {
                snapshot_relative: snapshot_relative.clone(),
                path: row.path.clone(),
                location: row.location.clone(),
                fragment: row.fragment.clone(),
            });
        }
    }
    if !issues.is_empty() {
        return CandidateValidation::Issues(issues);
    }

    let occurrences = candidate
        .references
        .into_iter()
        .map(|row| LegacyReferenceOccurrence::new(row.path, row.location, row.fragment))
        .collect();

    CandidateValidation::Valid(ValidatedCandidateSnapshot { occurrences })
}
