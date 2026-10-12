use super::super::WorthQueryApplicationAttemptDenial as SelfDenial;
use super::{
    adjacency, indexed_entity_selection, workflow_definition_predecessor, workflow_history_basis,
    workflow_instance_capacity, WorthQueryApplicationObservedFact,
};

impl WorthQueryApplicationObservedFact {
    pub(crate) fn remains_equal_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<bool, super::super::WorthQueryApplicationAttemptDenial> {
        let check_request = || {
            execution.consult().map_err(|stop| {
                SelfDenial::query_read_denial(
                    worth_relational::facade::runtime::QueryReadExecutionStop::Request(stop),
                    "decision fact comparison",
                )
            })
        };
        check_request()?;
        let equal = match self {
            Self::RetiredOutputEntity { entity_id, kind, created_at, deleted_at, .. } => super::retirement_equality::remains_equal(runtime, snapshot, *entity_id, *kind, *created_at, *deleted_at),
            Self::SourceEntity { entity_id } => match runtime.read_truth().project_snapshot(snapshot) {
                Some(view) => {
                    view.entity_record_with_projection_scope(execution,
                        *entity_id,
                        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                        |record| {
                            Some(
                                record.lifecycle()
                                    == worth_relational::facade::storage::RecordLifecycleState::Live,
                            )
                        },
                    ).map_err(|cause| SelfDenial::relational_read_stopped(cause, "source entity comparison"))? == Some(true)
                },
                None => false,
            },
            Self::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => runtime
                .read_truth()
                .exact_snapshot_entity_aspect_version(snapshot, *entity_id, aspect)
                == Some(*native_revision),
            Self::SourceFieldRevision { entity_id, locator, native_revision } =>
                if let Some(expected) = native_revision {
                    match runtime.read_truth().project_snapshot(snapshot) {
                        Some(view) => view.entity_field_revision(execution, *entity_id, locator)
                            .map_err(|cause| SelfDenial::relational_read_stopped(cause, "source field comparison"))? == Some(*expected),
                        None => false,
                    }
                } else { false },
            Self::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision,
                comparison_work_limit,
                ..
            } => match runtime.read_truth().project_snapshot(snapshot) {
                Some(view) => {
                    match view.bounded_adjacency_structural_revision(execution,
                        *anchor,
                        *relation_kind,
                        *direction,
                        *comparison_work_limit,
                    ) {
                        Ok(current) => current.revision() == *native_revision,
                        Err(worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::Execution(cause)) =>
                            return Err(SelfDenial::relational_read_stopped(cause, "source adjacency comparison")),
                        // A missing basis or insufficient declared comparison
                        // bound cannot prove equality. Request failures stay typed.
                        Err(
                            worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::WorkBudgetExceeded
                            | worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::AnchorUnavailable
                            | worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::BasisUnavailable,
                        ) => false,
                    }
                },
                None => false,
            },
            Self::Entity {
                entity_id, kind, ..
            } => runtime
                .read_truth()
                .exact_snapshot_live_entity_kind_status(snapshot, *entity_id)
                == Some(Some(*kind)),
            Self::Field {
                entity_id,
                kind,
                locator,
                value,
                ..
            } => super::super::observation::observe_field_value(
                runtime, execution, snapshot, *entity_id, *kind, locator,
            )?
            .is_some_and(|current| current == *value),
            Self::AbsentField {
                entity_id,
                kind,
                locator,
            } => matches!(
                super::super::observation::observe_field(runtime, execution, snapshot, *entity_id, *kind, locator)?,
                Some(super::super::observation::WorthQueryApplicationFieldObservation::Absent)
            ),
            Self::Relation {
                relation_kind,
                from,
                to,
                matching_relations,
                ..
            } => super::super::observation::exact_relations(
                runtime,
                execution,
                snapshot,
                *relation_kind,
                *from,
                *to,
            )
            ? == *matching_relations,
            Self::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units,
                relations,
                ..
            } => adjacency::remains_equal(
                runtime,
                execution,
                snapshot,
                *relation_kind,
                *anchor,
                *direction,
                *maximum_work_units,
                relations,
            )?,
            Self::IndexedEntitySelection {
                index_id,
                definition,
                entity_kind,
                locator,
                value,
                candidate_limit,
                candidates,
            } => indexed_entity_selection::remains_equal(
                runtime,
                execution,
                snapshot,
                *index_id,
                definition,
                *entity_kind,
                locator,
                value,
                *candidate_limit,
                candidates,
            )?,
            Self::WorkflowDefinitionPredecessor {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => workflow_definition_predecessor::remains_equal(
                runtime,
                execution,
                snapshot,
                *relation_kind,
                *lineage,
                *expected_definition,
                *maximum_work_units,
            )?,
            Self::WorkflowDefinitionCurrent {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => workflow_definition_predecessor::remains_equal(
                runtime,
                execution,
                snapshot,
                *relation_kind,
                Some(*lineage),
                Some(*expected_definition),
                *maximum_work_units,
            )?,
            Self::WorkflowInstanceCapacity {
                relation_kind,
                lineage,
                maximum_instances,
                instances,
            } => workflow_instance_capacity::remains_equal(
                runtime,
                execution,
                snapshot,
                *relation_kind,
                *lineage,
                *maximum_instances,
                instances,
            )?,
            Self::WorkflowHistoryBasis {
                maximum_transitions,
                transition_count,
                snapshot: observed,
                ..
            } => workflow_history_basis::remains_equal(
                runtime,
                snapshot,
                observed,
                *maximum_transitions,
                *transition_count,
            ),
        };
        check_request()?;
        Ok(equal)
    }
}
