//! Full derived-selection validation at checkpoint import, never ordinary maintenance.
use super::super::projected_field_values::*;
use crate::contracts::durability::data::recovery_errors::{DurabilityError, DurabilityErrorOwnerPort as _};
use crate::contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::contracts::indexes::data::basis_checkpoint::IndexBasisCheckpoint;
use crate::runtime::state::subsystems::indexing::IndexingState;
use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use std::collections::BTreeMap;
use worth_relational_branch::internal_port::branch::root_snapshot_custody::VisibilityProjectionViewOwnerPort as _;
use worth_relational_branch::internal_port::indexes::data::relation_join_index::RelationJoinIndexOwnerPort as _;

#[derive(Debug)]
pub struct CheckpointIndexProjections {
    context:
        worth_relational_branch::internal_port::visibility::materialization::read_records::projection::checkpoint_context::CheckpointProjectionContext,
}

impl CheckpointIndexProjections {
    pub fn from_context(
        context: worth_relational_branch::internal_port::visibility::materialization::read_records::projection::checkpoint_context::CheckpointProjectionContext,
    ) -> Self {
        Self { context }
    }

    pub fn context(
        &self,
    ) -> &worth_relational_branch::internal_port::visibility::materialization::read_records::projection::checkpoint_context::CheckpointProjectionContext
{
        &self.context
    }
}

pub fn verify_checkpoint_index_bases(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexingState,
    bases: &[IndexBasisCheckpoint],
    projections: &CheckpointIndexProjections,
    roots: &BTreeMap<
        crate::contracts::history::data::coordinates::CommitId,
        std::sync::Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
    >,
) -> Result<(), RelationalRecoveryStop> {
    crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
        for basis in bases {
            let root = roots
                .get(&basis.commit)
                .ok_or_else(|| corrupt("index basis has no selected checkpoint root"))?;
            let selection = root.admit_checkpoint_projection(lane, execution, &basis.branch)?;
            let projection = projections.context.project(lane, selection);
            verify_basis(lane, execution, indexes, basis, projection.view())?;
        }
        Ok(())
    })
}
pub fn verify_basis(
    lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<
        '_,
    >,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexingState,
    basis: &IndexBasisCheckpoint,
    projection: &VisibilityProjectionView,
) -> Result<(), RelationalRecoveryStop> {
    let lease = match execution.allocation_policy() {
        worth_execution::ExecutionAllocationPolicy::Execution(lease) => Some(lease),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation => None,
    };
    let work = crate::contracts::execution::request_work::RequestWorkBudget::new();
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
            let crate::contracts::indexes::data::execution_denial::DerivedIndexExecutionDenialKind::Cause(cause) = denial.kind;
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
            use worth_relational_branch::internal_port::indexes::data::DerivedIndexEntries as E;
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
