//! Full derived-selection validation at checkpoint import, never ordinary maintenance.
use super::super::projected_field_values::*;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::indexes::data::IndexBasisCheckpoint;
use crate::runtime::{IndexingState, RelationalRuntime, VisibilityProjectionView};
use std::collections::BTreeMap;

mod cold_images;
#[derive(Debug)]
pub(crate) struct CheckpointIndexProjections {
    context: crate::runtime::CheckpointProjectionContext,
}

pub(crate) fn checkpoint_index_projections(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    runtime: &RelationalRuntime,
) -> CheckpointIndexProjections {
    CheckpointIndexProjections {
        context: crate::runtime::CheckpointProjectionContext::capture(lane, runtime),
    }
}

pub(crate) fn verify_checkpoint_index_bases(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexingState,
    bases: &[IndexBasisCheckpoint],
    projections: &CheckpointIndexProjections,
    roots: &BTreeMap<
        crate::history::data::CommitId,
        std::sync::Arc<crate::branch::RelationalBranchRoot>,
    >,
) -> Result<(), RelationalRecoveryStop> {
    crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
        for basis in bases {
            let root = roots
                .get(&basis.commit)
                .ok_or_else(|| corrupt("index basis has no selected checkpoint root"))?;
            let projection =
                projections
                    .context
                    .project(lane, execution, &basis.branch, root.clone())?;
            verify_basis(lane, execution, indexes, basis, projection.view())?;
        }
        Ok(())
    })
}
fn verify_basis(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexingState,
    basis: &IndexBasisCheckpoint,
    projection: &VisibilityProjectionView,
) -> Result<(), RelationalRecoveryStop> {
    let lease = match execution.allocation_policy() {
        worth_execution::ExecutionAllocationPolicy::Execution(lease) => Some(lease),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation => None,
    };
    let work = crate::execution::RequestWorkBudget::new();
    let root = projection
        .selected_root()
        .ok_or_else(|| corrupt("index checkpoint root is unavailable"))?;
    if projection.version_id() != basis.version
        || root.commit_id() != Some(basis.commit)
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
            let crate::indexes::data::DerivedIndexExecutionDenialKind::Cause(cause) = denial.kind;
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
            return Err(
                corrupt("checkpoint index selection differs from its authoritative root").into(),
            );
        }
    }
    Ok(())
}
fn corrupt(detail: &str) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
