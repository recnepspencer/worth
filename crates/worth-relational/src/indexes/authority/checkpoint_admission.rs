//! Full derived-selection validation at checkpoint import, never ordinary maintenance.
use super::super::projected_field_values::*;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::identity::data::VersionId;
use crate::indexes::data::IndexBasisCheckpoint;
use crate::runtime::{IndexingState, RelationalRuntime, VisibilityProjectionView};
use std::collections::BTreeMap;

pub(crate) type CheckpointIndexProjections = BTreeMap<VersionId, VisibilityProjectionView>;

pub(crate) fn checkpoint_index_projections(
    _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    runtime: &RelationalRuntime,
) -> CheckpointIndexProjections {
    use crate::visibility::snapshot_states::{SnapshotStateBasis, VisibilitySnapshotBasis};
    // Cold import captures actual retained roots, including selected historical
    // bases. This is a root inventory, never a timeline/envelope scan.
    runtime
        .history
        .current_root_storage_inventory()
        .into_iter()
        .chain(
            runtime
                .history
                .retired_root_storage_inventory()
                .into_iter()
                .map(|entry| entry.root),
        )
        .filter_map(|root| {
            let envelope = root.canonical_envelope()?;
            let branch = envelope.branch_context.clone();
            let version = envelope.commit.version_id;
            let view = VisibilityProjectionView::new(
                runtime,
                SnapshotStateBasis::Exact(VisibilitySnapshotBasis::from_prepared_root(
                    branch, root,
                )),
            );
            Some((version, view))
        })
        .collect()
}

pub(crate) fn verify_checkpoint_index_bases(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexingState,
    bases: &[IndexBasisCheckpoint],
    projections: &CheckpointIndexProjections,
) -> Result<(), RelationalRecoveryStop> {
    crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
        verify_in_reconstruction(lane, execution, indexes, bases, projections)
    })
}
fn verify_in_reconstruction(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexingState,
    bases: &[IndexBasisCheckpoint],
    projections: &CheckpointIndexProjections,
) -> Result<(), RelationalRecoveryStop> {
    let lease = match execution.allocation_policy() {
        worth_execution::ExecutionAllocationPolicy::Execution(lease) => Some(lease),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation => None,
    };
    let work = crate::execution::RequestWorkBudget::new();
    for basis in bases {
        let projection = projections
            .get(&basis.version)
            .ok_or_else(|| corrupt("index basis has no selected checkpoint root"))?;
        let root = projection
            .selected_root()
            .ok_or_else(|| corrupt("index checkpoint root is unavailable"))?;
        if root.commit_id() != Some(basis.commit)
            || root.schema_authority().schema_version() != basis.schema
        {
            return Err(corrupt("index basis mismatches selected checkpoint root").into());
        }
        let source = IndexProjectionSource::selected(projection);
        for (index, generation) in basis
            .bindings
            .iter()
            .map(|(index, generation)| (index, Some(generation)))
            .chain(basis.empty.iter().map(|index| (index, None)))
        {
            let definition = indexes
                .definition(*index)
                .ok_or_else(|| corrupt("index basis definition is unavailable"))?;
            let mut expected = super::execute_index_packets(
                lane,
                execution,
                &source,
                super::plan_index_packets(vec![definition.as_ref().clone()]),
                lease,
                Some(&work),
            )
            .map_err(|denial| {
                let crate::indexes::data::DerivedIndexExecutionDenialKind::Cause(cause) =
                    denial.kind;
                RelationalRecoveryStop::from(cause)
            })?;
            let expected = expected
                .pop()
                .and_then(|(_, result)| result.entries)
                .ok_or_else(|| corrupt("checkpoint index projection is unavailable"))?;
            let agrees = if let Some(generation) = generation {
                indexes
                    .checkpoint_generation(*generation)
                    .is_some_and(|generation| generation.entries == expected)
            } else {
                use crate::indexes::data::DerivedIndexEntries as E;
                match &expected {
                    E::EntityField(rows) => rows.is_empty(),
                    E::RelationField(rows) => rows.is_empty(),
                    E::RelatedEntityOrdering(rows) => rows.is_empty(),
                    E::RelationJoin(rows) => {
                        rows.is_empty()
                            && rows.legs().is_some_and(|legs| {
                                legs.left.relation_count() == 0 && legs.right.relation_count() == 0
                            })
                    }
                }
            };
            if !agrees {
                return Err(corrupt(
                    "checkpoint index selection differs from its authoritative root",
                )
                .into());
            }
        }
    }
    Ok(())
}
fn corrupt(detail: &str) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
