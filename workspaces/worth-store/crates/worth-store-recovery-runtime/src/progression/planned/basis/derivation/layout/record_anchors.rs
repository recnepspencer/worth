//! Replay projects record anchors over the selected root exactly as the
//! interrupted publications planned them.

use super::*;
use worth_store_physical_format::{
    PersistedPhysicalRecoveryOperation as Semantic, PersistedRecordIdentity, RecordAnchorTable,
};

/// The anchors the staged root selects: the selected root's table advanced by
/// each pending anchored append, in WAL order. An advance that does not apply
/// to the replayed table, or a verified drop of an anchored record, refuses.
pub(super) fn projected_record_anchors(
    selection: &PhysicalSourceSelection,
    pending: &PendingProjectionBasis<'_>,
    verified_drops: &[PersistedRecordIdentity],
) -> Result<RecordAnchorTable, ExecutionBasisDenial> {
    let mut anchors = selection.root().selected().manifest().record_anchors();
    for projection in &pending.projections {
        if let Semantic::RecordAnchorAdvanced(anchored) = projection.materialization().operation() {
            if anchored.candidate_root_generation() != pending.staging_generation {
                return Err(ExecutionBasisDenial::Invalid);
            }
            anchors = anchors
                .advanced(anchored.advance())
                .map_err(|_| ExecutionBasisDenial::Invalid)?;
        }
    }
    if anchors
        .iter()
        .any(|anchor| verified_drops.binary_search(&anchor.record()).is_ok())
    {
        return Err(ExecutionBasisDenial::Invalid);
    }
    Ok(anchors)
}
