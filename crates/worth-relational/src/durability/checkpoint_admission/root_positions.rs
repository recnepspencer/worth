//! Decode derived route selectors without constructing a root or publication.
use super::RelationalCheckpointAdmissionRefusal as Refusal;
use crate::durability::canonical_commit_artifact::borrowed_payload::{
    malformed, BorrowedPayload, Cursor,
};
use crate::history::data::{CommitId, RequiredOutputPublicationPositions};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(super) fn read(
    roots: &[u8],
    commits: &ExecutionOrderedMap<u64, usize>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionOrderedMap<CommitId, RequiredOutputPublicationPositions>, Refusal> {
    traversal(roots.len(), request)?;
    let roots = named_field(roots, "roots", &["roots", "empty", "schemas"], request)?;
    let mut positions = ExecutionOrderedMap::default();
    let mut stopped = None;
    traversal(roots.0.len(), request)?;
    let result = roots.elements(request, |row| {
        let result = read_row(row, commits, &mut positions, request);
        if let Err(cause) = result {
            stopped = Some(cause);
            return Err(malformed());
        }
        Ok(())
    });
    if let Some(cause) = stopped {
        return Err(cause);
    }
    result.map_err(projected_stop)?;
    Ok(positions)
}

fn read_row(
    row: BorrowedPayload<'_>,
    commits: &ExecutionOrderedMap<u64, usize>,
    positions: &mut ExecutionOrderedMap<CommitId, RequiredOutputPublicationPositions>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    traversal(row.0.len(), request)?;
    let mut fields = [None; 2];
    let mut index = 0;
    row.elements(request, |field| {
        let slot = fields.get_mut(index).ok_or_else(malformed)?;
        *slot = Some(field);
        index += 1;
        Ok(())
    })
    .map_err(projected_stop)?;
    let [Some(header), Some(selector)] = fields else {
        return Err(Refusal::InvalidProjectedMeaning);
    };
    traversal(header.0.len(), request)?;
    let commit = named_field(
        header.0,
        "commit",
        &["commit", "components", "schema", "regions"],
        request,
    )?;
    let mut scalar = Cursor::new(commit.0, request);
    let commit = CommitId(scalar.unsigned().map_err(projected_stop)?);
    scalar.finish().map_err(projected_stop)?;
    let Some(root_position) = commits.get(&commit.0).copied() else {
        return Err(Refusal::InvalidProjectedMeaning);
    };
    if positions.contains_key(&commit) {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let selector = RequiredOutputPublicationPositions::readmit_checkpoint(
        selector.0,
        &|commit| {
            commits
                .get(&commit.0)
                .is_some_and(|position| *position <= root_position)
        },
        request,
    )?;
    positions
        .insert_cloned_native_for_request(&commit, &selector, *request)
        .map_err(|cause| {
            super::refusal::native_stop(crate::durability::checkpoint_history::capture::map_stop(
                cause,
            ))
        })?;
    Ok(())
}

fn named_field<'a>(
    bytes: &'a [u8],
    selected: &str,
    fields: &[&str],
    request: &ExecutionRequest<'_, '_>,
) -> Result<BorrowedPayload<'a>, Refusal> {
    let mut cursor = Cursor::new(bytes, request);
    if cursor.map_length().map_err(projected_stop)? != fields.len() {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let mut result = None;
    for expected in fields {
        let key = cursor.string_bytes().map_err(projected_stop)?;
        if key.len() != expected.len() || key != expected.as_bytes() {
            return Err(Refusal::InvalidProjectedMeaning);
        }
        let value = cursor.value().map_err(projected_stop)?;
        if *expected == selected {
            result = Some(BorrowedPayload(value));
        }
    }
    cursor.finish().map_err(projected_stop)?;
    result.ok_or(Refusal::InvalidProjectedMeaning)
}

fn traversal(bytes: usize, request: &ExecutionRequest<'_, '_>) -> Result<(), Refusal> {
    crate::durability::artifact_payload_emission::request::account_work(request, bytes as u64)
        .map_err(super::refusal::native_stop)
}

pub(crate) fn projected_stop(
    cause: crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal,
) -> Refusal {
    match cause {
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Content(cause) => Refusal::Content(cause),
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(cause) => Refusal::Request(cause),
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Frame(_)
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::PayloadDecode { .. }
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::PositionDisagreement { .. }
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::CommitIdDisagreement { .. }
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::FieldText(_)
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::PositionAdmission(_)
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::NonCanonicalPayload => Refusal::InvalidProjectedMeaning,
    }
}
