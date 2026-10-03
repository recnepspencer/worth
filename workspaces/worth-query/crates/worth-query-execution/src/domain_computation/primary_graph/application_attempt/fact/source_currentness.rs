use super::WorthQueryApplicationObservedFact;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQuerySourceCurrentnessFailure {
    WorkBudgetExceeded,
    Unavailable,
}

impl WorthQueryApplicationObservedFact {
    /// Exact-root native probes have a known cost before they run. Callers
    /// with cumulative Work custody prepay this amount so an unavailable
    /// result cannot erase already performed lookup/hash work.
    pub(in crate::domain_computation::primary_graph) fn exact_probe_work(
        &self,
    ) -> Result<Option<usize>, WorthQuerySourceCurrentnessFailure> {
        match self {
            Self::Entity { .. } => Ok(Some(1)),
            Self::SourceAspectRevision { aspect, .. } => aspect
                .as_str()
                .len()
                .checked_add(1)
                .map(Some)
                .ok_or(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded),
            _ => Ok(None),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn source_currentness_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        maximum_work: usize,
    ) -> Result<(bool, usize), WorthQuerySourceCurrentnessFailure> {
        if maximum_work == 0 {
            return Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded);
        }
        match self {
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
                let work = self
                    .exact_probe_work()?
                    .expect("an aspect fact has exact native probe work");
                if work > maximum_work {
                    return Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded);
                }
                let current = runtime
                    .read_truth()
                    .exact_snapshot_entity_aspect_version(snapshot, *entity_id, aspect)
                    .ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?;
                Ok((current == *native_revision, work))
            }
            Self::SourceFieldRevision {
                entity_id,
                locator,
                native_revision,
            } => {
                let expected =
                    native_revision.ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?;
                let current = runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .and_then(|view| view.entity_field_revision(*entity_id, locator))
                    .ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)?;
                Ok((current == expected, 1))
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
                let comparison = view
                    .bounded_adjacency_structural_revision(
                        *anchor,
                        *relation_kind,
                        *direction,
                        (*comparison_work_limit).min(maximum_work),
                    )
                    .map_err(|_| WorthQuerySourceCurrentnessFailure::Unavailable)?;
                Ok((
                    comparison.revision() == *native_revision,
                    comparison.work_units(),
                ))
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
                let (relations, work) = super::adjacency::observe_adjacency_with_work(
                    runtime,
                    snapshot,
                    *relation_kind,
                    *from,
                    super::WorthQueryApplicationAdjacencyDirection::Outgoing,
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
                let (current, work) = super::adjacency::observe_adjacency_with_work(
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
            Self::IndexedEntitySelection {
                index_id,
                definition,
                entity_kind,
                locator,
                value,
                candidate_limit,
                candidates,
            } => {
                if candidate_limit
                    .checked_add(1)
                    .is_none_or(|needed| needed > maximum_work)
                {
                    return Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded);
                }
                super::indexed_entity_selection::remains_equal_with_work(
                    runtime,
                    snapshot,
                    *index_id,
                    definition,
                    *entity_kind,
                    locator,
                    value,
                    *candidate_limit,
                    candidates,
                )
                .ok_or(WorthQuerySourceCurrentnessFailure::Unavailable)
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
                let (current, work) = super::adjacency::observe_adjacency_with_work(
                    runtime,
                    snapshot,
                    *relation_kind,
                    *lineage,
                    super::WorthQueryApplicationAdjacencyDirection::Outgoing,
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
                let (current, work) = super::adjacency::observe_adjacency_with_work(
                    runtime,
                    snapshot,
                    *relation_kind,
                    *lineage,
                    super::WorthQueryApplicationAdjacencyDirection::Outgoing,
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
                let (current, work) = super::adjacency::observe_adjacency_with_work(
                    runtime,
                    snapshot,
                    *relation_kind,
                    *lineage,
                    super::WorthQueryApplicationAdjacencyDirection::Incoming,
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
                super::workflow_history_basis::remains_equal(
                    runtime,
                    snapshot,
                    observed,
                    *maximum_transitions,
                    *transition_count,
                ),
                1,
            )),
            // Decision facts are value observations, not native revisions.
            // Output lineage must first rebase them at the committed snapshot.
            Self::Field { .. } | Self::AbsentField { .. } => {
                Err(WorthQuerySourceCurrentnessFailure::Unavailable)
            }
        }
    }
}

fn map_adjacency_denial(
    denial: super::AdjacencyObservationDenial,
) -> WorthQuerySourceCurrentnessFailure {
    match denial {
        super::AdjacencyObservationDenial::SnapshotUnavailable => {
            WorthQuerySourceCurrentnessFailure::Unavailable
        }
        super::AdjacencyObservationDenial::WorkBudgetExceeded => {
            WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded
        }
    }
}
