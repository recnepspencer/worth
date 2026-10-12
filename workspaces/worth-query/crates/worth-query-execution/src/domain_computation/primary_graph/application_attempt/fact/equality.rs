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
        execution
            .check_live()
            .map_err(|stop| SelfDenial::request_read_stopped(stop, "decision fact comparison"))?;
        let equal = match self {
            Self::RetiredOutputEntity { entity_id, kind, created_at, deleted_at, .. } => super::retirement_equality::remains_equal(runtime, execution, snapshot, *entity_id, *kind, *created_at, *deleted_at),
            Self::SourceEntity { entity_id } => runtime
                .read_truth()
                .project_snapshot(snapshot, execution)
                .is_some_and(|view| {
                    view.entity_record_with_projection_scope(execution,
                        *entity_id,
                        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                        |record| {
                            Some(
                                record.lifecycle()
                                    == worth_relational::facade::storage::RecordLifecycleState::Live,
                            )
                        },
                    ) == Some(true)
                }),
            Self::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => runtime
                .read_truth()
                .exact_snapshot_entity_aspect_version(snapshot, *entity_id, aspect)
                == Some(*native_revision),
            Self::SourceFieldRevision { entity_id, locator, native_revision } =>
                native_revision.is_some_and(|expected| {
                    runtime.read_truth().project_snapshot(snapshot, execution)
                        .and_then(|view| view.entity_field_revision(execution, *entity_id, locator)) == Some(expected)
                }),
            Self::SourceAdjacencyRevision { relation_kind, anchor, direction, native_revision, comparison_work_limit, .. } => {
                match runtime.read_truth().project_snapshot(snapshot, execution) {
                    None => false,
                    Some(view) => match view.bounded_adjacency_structural_revision(execution, *anchor, *relation_kind, *direction, *comparison_work_limit) {
                        Ok(current) => current.revision() == *native_revision,
                        Err(worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::Execution(cause)) => return Err(SelfDenial::relational_read_stopped(cause, "structural revision comparison")),
                        Err(worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::WorkBudgetExceeded
                            | worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::AnchorUnavailable
                            | worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::BasisUnavailable) => false,
                    },
                }
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
            )
            .is_some_and(|current| current == *value),
            Self::AbsentField {
                entity_id,
                kind,
                locator,
            } => matches!(
                super::super::observation::observe_field(runtime, execution, snapshot, *entity_id, *kind, locator),
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
                execution,
                snapshot,
                observed,
                *maximum_transitions,
                *transition_count,
            ),
        };
        execution
            .check_live()
            .map_err(|stop| SelfDenial::request_read_stopped(stop, "decision fact comparison"))?;
        Ok(equal)
    }
}
