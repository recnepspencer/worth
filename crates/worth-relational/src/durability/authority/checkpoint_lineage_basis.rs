//! Checkpoint lineage totals come from admitted index facts, never body residency.
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalCheckpointManifest,
};
use crate::history::RelationalCommitArtifact;
use crate::lineage::data::LineageCheckpointDigestBasis;
use std::sync::Arc;

pub(super) fn capture(
    artifacts: &[Arc<RelationalCommitArtifact>],
    prior: Option<&RelationalCheckpointManifest>,
) -> Result<LineageCheckpointDigestBasis, DurabilityError> {
    let mut basis = prior.map_or_else(
        || LineageCheckpointDigestBasis::new(0, Vec::new(), 0, 0),
        |prior| prior.lineage_basis.clone(),
    );
    let mut rows = Vec::new();
    for artifact in artifacts {
        let cell = artifact.envelope_residency();
        if let Some(entry) = cell.journal_index()
            && prior.is_some_and(|prior| entry.position <= prior.covered_position)
        {
            continue;
        }
        let row = cell.checkpoint_lineage().ok_or_else(|| {
            unavailable("checkpoint has no admitted lineage index facts for an uncovered commit")
        })?;
        if prior.is_none_or(|prior| row.0 > prior.covered_position) {
            rows.push(row);
        }
    }
    rows.sort_by_key(|(position, _)| *position);
    for (_, delta) in rows {
        basis.published_lineage_commit_count = add(
            basis.published_lineage_commit_count,
            delta.published_lineage_commit_count,
        )?;
        basis.published_lineage_event_count = add(
            basis.published_lineage_event_count,
            delta.published_lineage_event_count,
        )?;
        basis.published_lineage_decision_count = add(
            basis.published_lineage_decision_count,
            delta.published_lineage_decision_count,
        )?;
        basis
            .canonical_published_event_ids
            .extend(delta.canonical_published_event_ids);
    }
    Ok(basis)
}
fn add(total: usize, delta: usize) -> Result<usize, DurabilityError> {
    total
        .checked_add(delta)
        .ok_or_else(|| unavailable("checkpoint lineage totals overflow"))
}
fn unavailable(detail: &str) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
