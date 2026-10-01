use worth_store_physical_format::{BlobTreeEntryV1, BlobTreeNodeV1, PersistedRecordIdentity};

use super::super::super::tree::{BlobTreeBuildFailure, BlobTreeBuilder};
use super::super::node_writer::BlobNodeWriter;
use super::{
    claims::SelectedResumeClaim,
    record_read::{read_record_into, RESUME_FRAME_BYTES},
    selection::SelectedResumeState,
    BlobResumeFailure, RetainedBlobNodes,
};

enum DryWriteFailure {
    Missing,
    Invalid(BlobResumeFailure),
}

/// Existing selected nodes must be a prefix of the real producer's emission
/// order. A failed append poisons that producer, so a later node cannot skip
/// an earlier missing node. This pass performs no mutations.
pub(super) fn validate_existing_nodes(
    state: &mut SelectedResumeState,
) -> Result<(), BlobResumeFailure> {
    let mut tree = BlobTreeBuilder::new(state.declaration.store(), state.declaration.session());
    let mut reused = 0_usize;
    let expected = state
        .claims
        .iter()
        .filter(|claim| matches!(claim, SelectedResumeClaim::Node { .. }))
        .count();
    let mut bytes = 0_u64;
    for index in 0..state.claims.len() {
        let Some((_, record, digest, count)) = state.claims[index].chunk() else {
            continue;
        };
        bytes += count;
        let result = tree.push(entry(record, digest, count)?, &mut |node, ordinal| {
            verify_selected_node(state, node, ordinal, &mut reused)
        });
        if !dry_completed(result)? {
            return require_all_nodes(reused, expected);
        }
    }
    if bytes == state.declaration.declared_bytes() {
        let result = tree
            .finish(&mut |node, ordinal| verify_selected_node(state, node, ordinal, &mut reused));
        dry_completed(result.map(|_| ()))?;
    }
    require_all_nodes(reused, expected)
}

fn dry_completed(
    result: Result<(), BlobTreeBuildFailure<DryWriteFailure>>,
) -> Result<bool, BlobResumeFailure> {
    match result {
        Ok(()) => Ok(true),
        Err(BlobTreeBuildFailure::Write(DryWriteFailure::Missing)) => Ok(false),
        Err(BlobTreeBuildFailure::Write(DryWriteFailure::Invalid(failure))) => Err(failure),
        Err(BlobTreeBuildFailure::Format(denial)) => Err(BlobResumeFailure::Format(denial)),
        Err(BlobTreeBuildFailure::TooDeep) => Err(BlobResumeFailure::TreeConflict),
    }
}

fn require_all_nodes(reused: usize, expected: usize) -> Result<(), BlobResumeFailure> {
    if reused == expected {
        Ok(())
    } else {
        Err(BlobResumeFailure::TreeConflict)
    }
}

fn verify_selected_node(
    state: &mut SelectedResumeState,
    node: BlobTreeNodeV1,
    ordinal: u64,
    reused: &mut usize,
) -> Result<PersistedRecordIdentity, DryWriteFailure> {
    let (record, _) = find_node(&state.claims, ordinal).ok_or(DryWriteFailure::Missing)?;
    let used = read_record_into(
        &state.reader,
        record,
        &mut state.scratch,
        RESUME_FRAME_BYTES,
    )
    .map_err(DryWriteFailure::Invalid)?;
    if node.encode().as_slice() != &state.scratch[..used] {
        return Err(DryWriteFailure::Invalid(BlobResumeFailure::TreeConflict));
    }
    *reused += 1;
    Ok(record)
}

pub(super) fn missing_full_nodes(state: &SelectedResumeState) -> bool {
    let chunks = chunk_count(&state.claims);
    let expected: u64 = (0..BlobTreeBuilder::maximum_levels())
        .map(|level| BlobTreeBuilder::full_node_count(chunks, level as u8))
        .sum();
    let selected = state
        .claims
        .iter()
        .filter(|claim| match claim {
            SelectedResumeClaim::Node { ordinal, .. } => is_full_node(*ordinal, chunks),
            _ => false,
        })
        .count() as u64;
    selected != expected
}

/// Runs only after every old claim, payload and node passed readmission.
/// Full selected nodes are reused; missing full nodes use the ordinary writer.
pub(super) fn restore(
    state: &mut SelectedResumeState,
    writer: &mut BlobNodeWriter<'_, '_>,
) -> Result<(BlobTreeBuilder, RetainedBlobNodes), BlobResumeFailure> {
    let mut tree = BlobTreeBuilder::new(state.declaration.store(), state.declaration.session());
    for claim in &state.claims {
        let Some((_, record, digest, bytes)) = claim.chunk() else {
            continue;
        };
        tree.push(entry(record, digest, bytes)?, &mut |node, ordinal| {
            if let Some((record, digest)) = find_node(&state.claims, ordinal) {
                if node.frame_digest() != digest {
                    return Err(BlobResumeFailure::TreeConflict);
                }
                state.observation.reused_nodes += 1;
                return Ok(record);
            }
            writer
                .write(node, ordinal)
                .map_err(BlobResumeFailure::Ingest)
        })
        .map_err(|failure| match failure {
            BlobTreeBuildFailure::Write(failure) => failure,
            BlobTreeBuildFailure::Format(denial) => BlobResumeFailure::Format(denial),
            BlobTreeBuildFailure::TooDeep => BlobResumeFailure::TreeConflict,
        })?;
    }
    let chunks = chunk_count(&state.claims);
    let mut retained = RetainedBlobNodes::empty();
    for claim in &state.claims {
        if let SelectedResumeClaim::Node {
            ordinal,
            record,
            frame_digest,
        } = *claim
        {
            if !is_full_node(ordinal, chunks) {
                retained.retain(ordinal, record, frame_digest)?;
            }
        }
    }
    Ok((tree, retained))
}

fn is_full_node(ordinal: u64, chunks: u64) -> bool {
    let level = (ordinal >> 56) as u8;
    let index = ordinal & ((1_u64 << 56) - 1);
    index < BlobTreeBuilder::full_node_count(chunks, level)
}

fn chunk_count(claims: &[SelectedResumeClaim]) -> u64 {
    claims
        .iter()
        .filter(|claim| claim.chunk().is_some())
        .count() as u64
}

fn find_node(
    claims: &[SelectedResumeClaim],
    ordinal: u64,
) -> Option<(PersistedRecordIdentity, [u8; 32])> {
    let index = claims
        .binary_search_by_key(&(1, ordinal), |claim| claim.key())
        .ok()?;
    match claims[index] {
        SelectedResumeClaim::Node {
            record,
            frame_digest,
            ..
        } => Some((record, frame_digest)),
        _ => None,
    }
}

fn entry(
    record: PersistedRecordIdentity,
    digest: [u8; 32],
    bytes: u64,
) -> Result<BlobTreeEntryV1, BlobResumeFailure> {
    BlobTreeEntryV1::new(digest, record, bytes).map_err(BlobResumeFailure::Format)
}
