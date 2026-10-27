use worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial;

use super::{WorthQueryApplicationObservedFact, WorthQuerySourceCurrentnessFailure};

impl WorthQueryApplicationObservedFact {
    pub(super) fn comparison(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<bool, WorthQuerySourceCurrentnessFailure> {
        match self {
            Self::SourcePublication { observation } => {
                observation.remains_equal(runtime, execution, snapshot)
            }
            Self::RetiredOutputEntity {
                entity_id,
                kind,
                created_at,
                deleted_at,
                ..
            } => Ok(super::super::retirement_equality::remains_equal(
                runtime,
                execution,
                snapshot,
                *entity_id,
                *kind,
                *created_at,
                *deleted_at,
            )
            .map_err(WorthQuerySourceCurrentnessFailure::Execution)?),
            Self::SourceEntity { entity_id } => {
                let view = runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?;
                Ok(view
                    .entity_record_with_projection_scope(
                        execution,
                        *entity_id,
                        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                        |record| {
                            Some(record.lifecycle()
                        == worth_relational::facade::storage::RecordLifecycleState::Live)
                        },
                    )
                    .map_err(WorthQuerySourceCurrentnessFailure::Execution)?
                    == Some(true))
            }
            Self::AspectPresence {
                entity_id,
                kind,
                aspect,
                present,
            } => Ok(super::super::super::observation::observe_aspect_presence(
                runtime, execution, snapshot, *entity_id, *kind, aspect,
            )
            .map_err(WorthQuerySourceCurrentnessFailure::Execution)?
                == Some(*present)),
            Self::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => {
                // The native symbol lookup hashes initialized aspect text.
                // Admit it before borrowing the exact selected root.
                let Some(current) = runtime
                    .read_truth()
                    .read_exact_snapshot_entity_aspect_version(
                        execution, snapshot, *entity_id, aspect,
                    )
                    .map_err(WorthQuerySourceCurrentnessFailure::Execution)?
                else {
                    return unanswered_probe(runtime, execution, snapshot, *entity_id);
                };
                Ok(current == *native_revision)
            }
            Self::SourceFieldRevision {
                entity_id,
                locator,
                native_revision,
            } => {
                let expected =
                    native_revision.ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?;
                let Some(current) = runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .map(|view| view.entity_field_revision(execution, *entity_id, locator))
                    .transpose()
                    .map_err(WorthQuerySourceCurrentnessFailure::Execution)?
                    .flatten()
                else {
                    return unanswered_probe(runtime, execution, snapshot, *entity_id);
                };
                Ok(current == expected)
            }
            Self::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision,
                comparison_work_limit,
                ..
            } => {
                let view = runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?;
                // The owner-index lookup costs one unit whatever the fact's
                // recorded limit.
                match view.bounded_adjacency_structural_revision(
                    execution,
                    *anchor,
                    *relation_kind,
                    *direction,
                    (*comparison_work_limit).min(1),
                ) {
                    Ok(comparison) => Ok(comparison.revision() == *native_revision),
                    Err(AdjacencyStructuralRevisionDenial::Execution(cause)) => {
                        Err(WorthQuerySourceCurrentnessFailure::Execution(cause))
                    }
                    Err(AdjacencyStructuralRevisionDenial::AnchorUnavailable) => {
                        unanswered_probe(runtime, execution, snapshot, *anchor)
                    }
                    Err(AdjacencyStructuralRevisionDenial::WorkBudgetExceeded) => {
                        Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)
                    }
                    Err(AdjacencyStructuralRevisionDenial::BasisUnavailable) => {
                        Err(WorthQuerySourceCurrentnessFailure::Unavailable)
                    }
                }
            }
            Self::Entity { entity_id, kind } => Ok(runtime
                .read_truth()
                .read_exact_snapshot_live_entity_kind_status(execution, snapshot, *entity_id)
                .map_err(WorthQuerySourceCurrentnessFailure::Execution)?
                .ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?
                == Some(*kind)),
            Self::Relation { .. } => {
                unreachable!("relation comparison charges the original admission while scanning")
            }
            Self::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units,
                relations,
            } => {
                let (current, _) = super::super::adjacency::observe_adjacency_with_work(
                    runtime,
                    execution,
                    snapshot,
                    *relation_kind,
                    *anchor,
                    *direction,
                    Some(*maximum_work_units),
                )
                .map_err(map_adjacency_denial)?;
                Ok(current == *relations)
            }
            Self::IndexedEntitySelection {
                candidate_limit, ..
            } => {
                let mut remaining = candidate_limit.saturating_add(1);
                super::super::indexed_entity_selection::currentness_with_remaining(
                    self,
                    runtime,
                    execution,
                    snapshot,
                    &mut remaining,
                )
            }
            Self::WorkflowDefinitionPredecessor {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => {
                let Some(lineage) = lineage else {
                    return Ok(false);
                };
                let (current, _) = super::super::adjacency::observe_adjacency_with_work(
                    runtime,
                    execution,
                    snapshot,
                    *relation_kind,
                    *lineage,
                    super::super::WorthQueryApplicationAdjacencyDirection::Outgoing,
                    Some(*maximum_work_units),
                )
                .map_err(map_adjacency_denial)?;
                let equal = match expected_definition {
                    Some(expected) => {
                        matches!(current.as_slice(), [relation] if relation.to == *expected)
                    }
                    None => current.is_empty(),
                };
                Ok(equal)
            }
            Self::WorkflowDefinitionCurrent {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => {
                let (current, _) = super::super::adjacency::observe_adjacency_with_work(
                    runtime,
                    execution,
                    snapshot,
                    *relation_kind,
                    *lineage,
                    super::super::WorthQueryApplicationAdjacencyDirection::Outgoing,
                    Some(*maximum_work_units),
                )
                .map_err(map_adjacency_denial)?;
                Ok(matches!(current.as_slice(), [relation] if relation.to == *expected_definition))
            }
            Self::WorkflowInstanceCapacity {
                relation_kind,
                lineage,
                maximum_instances,
                instances,
            } => {
                let bound = maximum_instances
                    .checked_mul(2)
                    .and_then(|count| count.checked_add(1))
                    .ok_or(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)?;
                let (current, _) = super::super::adjacency::observe_adjacency_with_work(
                    runtime,
                    execution,
                    snapshot,
                    *relation_kind,
                    *lineage,
                    super::super::WorthQueryApplicationAdjacencyDirection::Incoming,
                    Some(bound),
                )
                .map_err(map_adjacency_denial)?;
                Ok(instances.len() < *maximum_instances && current == *instances)
            }
            Self::WorkflowHistoryBasis {
                requirement,
                transition_count,
                snapshot: observed,
                ..
            } => Ok(super::super::workflow_history_basis::remains_equal(
                runtime,
                snapshot,
                observed,
                *requirement,
                *transition_count,
            )),
            Self::Field { .. } | Self::AbsentField { .. } | Self::OptionalAspectAbsent { .. } => {
                Err(WorthQuerySourceCurrentnessFailure::Unavailable)
            }
        }
    }
}

/// A native probe that found nothing still answers when its entity is no
/// longer live at this snapshot: the fact changed. So no fact's answer
/// depends on the entity fact beside it being read first. Any other silence
/// is a snapshot that could not be read.
// A selected fixed-key read still admits its region and preserves storage failure.
fn unanswered_probe(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: worth_relational::facade::identity::EntityId,
) -> Result<bool, WorthQuerySourceCurrentnessFailure> {
    match runtime
        .read_truth()
        .read_exact_snapshot_live_entity_kind_status(execution, snapshot, entity)
        .map_err(WorthQuerySourceCurrentnessFailure::Execution)?
    {
        Some(None) => Ok(false),
        _ => Err(WorthQuerySourceCurrentnessFailure::Unavailable),
    }
}

fn map_adjacency_denial(
    denial: super::super::AdjacencyObservationDenial,
) -> WorthQuerySourceCurrentnessFailure {
    match denial {
        super::super::AdjacencyObservationDenial::Execution(cause) => WorthQuerySourceCurrentnessFailure::Execution(cause),
        super::super::AdjacencyObservationDenial::Interrupted(event) => {
            WorthQuerySourceCurrentnessFailure::Execution(match event.interruption() {
                worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
                worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed,
            })
        }
        super::super::AdjacencyObservationDenial::SnapshotUnavailable => {
            WorthQuerySourceCurrentnessFailure::Unavailable
        }
        super::super::AdjacencyObservationDenial::WorkBudgetExceeded => {
            WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded
        }
    }
}
