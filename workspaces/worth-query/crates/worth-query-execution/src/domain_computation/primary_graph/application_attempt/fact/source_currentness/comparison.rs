use worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial;

use super::{WorthQueryApplicationObservedFact, WorthQuerySourceCurrentnessFailure};

impl WorthQueryApplicationObservedFact {
    pub(super) fn comparison(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        maximum_work: usize,
    ) -> Result<(bool, usize), WorthQuerySourceCurrentnessFailure> {
        if maximum_work == 0 {
            return Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded);
        }
        match self {
            Self::RetiredOutputEntity { .. } => Ok((self.remains_equal_in(runtime, snapshot), 1)),
            Self::SourceEntity { entity_id } => {
                let view = runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?;
                Ok((
                    view.entity_record_with_projection_scope(
                        *entity_id,
                        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                        |record| {
                            Some(record.lifecycle()
                        == worth_relational::facade::storage::RecordLifecycleState::Live)
                        },
                    ) == Some(true),
                    1,
                ))
            }
            Self::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => {
                // The native symbol lookup hashes initialized aspect text.
                // Admit it before borrowing the exact selected root.
                let work = aspect
                    .as_str()
                    .len()
                    .checked_add(1)
                    .filter(|work| *work <= maximum_work)
                    .ok_or(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)?;
                let Some(current) = runtime
                    .read_truth()
                    .exact_snapshot_entity_aspect_version(snapshot, *entity_id, aspect)
                else {
                    return unanswered_probe(runtime, snapshot, *entity_id, work, maximum_work);
                };
                Ok((current == *native_revision, work))
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
                    .and_then(|view| view.entity_field_revision(*entity_id, locator))
                else {
                    return unanswered_probe(runtime, snapshot, *entity_id, 1, maximum_work);
                };
                Ok((current == expected, 1))
            }
            Self::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision,
                ..
            } => {
                let view = runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?;
                // The owner-index lookup costs one unit whatever the fact's
                // recorded limit.
                match view.bounded_adjacency_structural_revision(
                    *anchor,
                    *relation_kind,
                    *direction,
                    1,
                ) {
                    Ok(comparison) => Ok((
                        comparison.revision() == *native_revision,
                        comparison.work_units(),
                    )),
                    Err(AdjacencyStructuralRevisionDenial::AnchorUnavailable) => {
                        unanswered_probe(runtime, snapshot, *anchor, 1, maximum_work)
                    }
                    Err(AdjacencyStructuralRevisionDenial::WorkBudgetExceeded) => {
                        Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)
                    }
                    Err(AdjacencyStructuralRevisionDenial::BasisUnavailable) => {
                        Err(WorthQuerySourceCurrentnessFailure::Unavailable)
                    }
                }
            }
            Self::Entity { entity_id, kind } => Ok((
                runtime
                    .read_truth()
                    .exact_snapshot_live_entity_kind_status(snapshot, *entity_id)
                    .ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?
                    == Some(*kind),
                1,
            )),
            Self::Relation {
                relation_kind,
                from,
                to,
                matching_relations,
            } => {
                let (relations, work) = super::super::adjacency::observe_adjacency_with_work(
                    runtime,
                    snapshot,
                    *relation_kind,
                    *from,
                    super::super::WorthQueryApplicationAdjacencyDirection::Outgoing,
                    maximum_work.saturating_sub(1),
                )
                .map_err(map_adjacency_denial)?;
                let matching = relations
                    .iter()
                    .filter(|relation| relation.to == *to)
                    .map(|relation| relation.relation_id);
                Ok((matching.eq(matching_relations.iter().copied()), work))
            }
            Self::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units,
                relations,
            } => {
                let (current, work) = super::super::adjacency::observe_adjacency_with_work(
                    runtime,
                    snapshot,
                    *relation_kind,
                    *anchor,
                    *direction,
                    (*maximum_work_units).min(maximum_work.saturating_sub(1)),
                )
                .map_err(map_adjacency_denial)?;
                Ok((current == *relations, work))
            }
            Self::IndexedEntitySelection { .. } => {
                unreachable!("indexed comparisons settle their carried reservation directly")
            }
            Self::WorkflowDefinitionPredecessor {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => {
                let Some(lineage) = lineage else {
                    return Ok((false, 1));
                };
                let (current, work) = super::super::adjacency::observe_adjacency_with_work(
                    runtime,
                    snapshot,
                    *relation_kind,
                    *lineage,
                    super::super::WorthQueryApplicationAdjacencyDirection::Outgoing,
                    (*maximum_work_units).min(maximum_work.saturating_sub(1)),
                )
                .map_err(map_adjacency_denial)?;
                let equal = match expected_definition {
                    Some(expected) => {
                        matches!(current.as_slice(), [relation] if relation.to == *expected)
                    }
                    None => current.is_empty(),
                };
                Ok((equal, work))
            }
            Self::WorkflowDefinitionCurrent {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => {
                let (current, work) = super::super::adjacency::observe_adjacency_with_work(
                    runtime,
                    snapshot,
                    *relation_kind,
                    *lineage,
                    super::super::WorthQueryApplicationAdjacencyDirection::Outgoing,
                    (*maximum_work_units).min(maximum_work.saturating_sub(1)),
                )
                .map_err(map_adjacency_denial)?;
                Ok((
                    matches!(current.as_slice(), [relation] if relation.to == *expected_definition),
                    work,
                ))
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
                let (current, work) = super::super::adjacency::observe_adjacency_with_work(
                    runtime,
                    snapshot,
                    *relation_kind,
                    *lineage,
                    super::super::WorthQueryApplicationAdjacencyDirection::Incoming,
                    bound.min(maximum_work.saturating_sub(1)),
                )
                .map_err(map_adjacency_denial)?;
                Ok((
                    instances.len() < *maximum_instances && current == *instances,
                    work,
                ))
            }
            Self::WorkflowHistoryBasis {
                maximum_transitions,
                transition_count,
                snapshot: observed,
                ..
            } => Ok((
                super::super::workflow_history_basis::remains_equal(
                    runtime,
                    snapshot,
                    observed,
                    *maximum_transitions,
                    *transition_count,
                ),
                1,
            )),
            Self::Field { .. } | Self::AbsentField { .. } => {
                Err(WorthQuerySourceCurrentnessFailure::Unavailable)
            }
        }
    }
}

/// A native probe that found nothing still answers when its entity is no
/// longer live at this snapshot: the fact changed. So no fact's answer
/// depends on the entity fact beside it being read first. Any other silence
/// is a snapshot that could not be read.
fn unanswered_probe(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: worth_relational::facade::identity::EntityId,
    probe_work: usize,
    maximum_work: usize,
) -> Result<(bool, usize), WorthQuerySourceCurrentnessFailure> {
    let work = probe_work
        .checked_add(1)
        .filter(|work| *work <= maximum_work)
        .ok_or(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)?;
    match runtime
        .read_truth()
        .exact_snapshot_live_entity_kind_status(snapshot, entity)
    {
        Some(None) => Ok((false, work)),
        _ => Err(WorthQuerySourceCurrentnessFailure::Unavailable),
    }
}

fn map_adjacency_denial(
    denial: super::super::AdjacencyObservationDenial,
) -> WorthQuerySourceCurrentnessFailure {
    match denial {
        super::super::AdjacencyObservationDenial::SnapshotUnavailable => {
            WorthQuerySourceCurrentnessFailure::Unavailable
        }
        super::super::AdjacencyObservationDenial::WorkBudgetExceeded => {
            WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded
        }
    }
}
