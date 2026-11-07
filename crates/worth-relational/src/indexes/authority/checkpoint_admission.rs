//! Full derived-selection validation at checkpoint import, never ordinary maintenance.
use super::super::projected_field_values::*;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::indexes::data::IndexBasisCheckpoint;
use crate::runtime::{IndexingState, RelationalRuntime, VisibilityProjectionView};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(crate) type CheckpointIndexProjections =
    BTreeMap<(BranchId, VersionId), VisibilityProjectionView>;

pub(crate) fn checkpoint_index_projections(
    runtime: &RelationalRuntime,
) -> CheckpointIndexProjections {
    use crate::visibility::snapshot_states::{SnapshotStateBasis, VisibilitySnapshotBasis};
    runtime
        .history
        .branch_ids_snapshot()
        .into_iter()
        .filter_map(|branch| {
            let root = runtime.history.branch_cell(&branch)?.root()?;
            let version = root.canonical_envelope()?.commit.version_id;
            let view = VisibilityProjectionView::new(
                runtime,
                SnapshotStateBasis::Exact(VisibilitySnapshotBasis::from_prepared_root(
                    branch.clone(),
                    Arc::clone(&root),
                )),
            );
            Some(((branch, version), view))
        })
        .collect()
}

pub(crate) fn verify_checkpoint_index_bases(
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
            .get(&(basis.branch.clone(), basis.version))
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
