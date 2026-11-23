//! Full derived-selection validation at checkpoint import, never ordinary maintenance.
use super::super::projected_field_values::*;
use crate::durability::data::recovery_errors::DurabilityError;
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::indexes::data::IndexBasisCheckpoint;
use crate::runtime::state::subsystems::indexing::IndexingState;
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct CheckpointIndexProjections {
    context:
        crate::visibility::materialization::read_records::projection::CheckpointProjectionContext,
}

impl CheckpointIndexProjections {
    pub(crate) fn from_context(
        context: crate::visibility::materialization::read_records::projection::CheckpointProjectionContext,
    ) -> Self {
        Self { context }
    }

    pub(crate) fn context(
        &self,
    ) -> &crate::visibility::materialization::read_records::projection::CheckpointProjectionContext
    {
        &self.context
    }
}

pub(crate) fn verify_checkpoint_index_bases(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexingState,
    bases: &[IndexBasisCheckpoint],
    projections: &CheckpointIndexProjections,
    roots: &BTreeMap<
        crate::history::data::CommitId,
        std::sync::Arc<crate::branch::root::RelationalBranchRoot>,
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
pub(crate) fn verify_basis(
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
    let work = crate::execution::request_work::RequestWorkBudget::new();
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
    DurabilityError::corrupt_checkpoint(detail)
}
