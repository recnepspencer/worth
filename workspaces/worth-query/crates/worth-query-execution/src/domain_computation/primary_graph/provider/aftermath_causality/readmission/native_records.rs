//! Read immutable canonical rows and retain the actual snapshot on release refusal.
use super::*;
pub(super) fn read(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryAftermathCausalityLayout,
    basis: &AdmittedRelationalBranchBasis,
    owner: WorldOwnerCoordinate,
) -> Result<Vec<WorthQueryCanonicalCorrectionFact>, Denial> {
    let view = native
        .read_truth()
        .project_observation(&basis.observation())
        .map_err(|cause| source_denial(format!("correction Native observation: {cause:?}")))?;
    let population = view
        .bounded_entities_of_kind(execution, layout.entity_kind, usize::MAX)
        .map_err(crate::domain_computation::primary_graph::bootstrap::request_read_denial::read)
        .map_err(Denial::Source)?
        .map_err(Denial::Population)?;
    if population.records().is_empty() {
        return Ok(Vec::new());
    }
    let snapshot = native
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .map_err(Denial::Snapshot)?;
    let result = population
        .records()
        .iter()
        .map(|row| {
            canonical_record::read(native, &snapshot, layout, execution, row.entity_id, owner)
                .map_err(|cause| Denial::Record {
                    entity: row.entity_id,
                    kind: layout.entity_kind,
                    cause,
                })
        })
        .collect();
    match native.snapshots().release_snapshot(&snapshot) {
        Ok(_) => result,
        Err(cause) => Err(Denial::SnapshotRelease {
            snapshot,
            cause,
            preceding: result.err().map(Box::new),
        }),
    }
}
