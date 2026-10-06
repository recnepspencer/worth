use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial;

use super::{FactMovement, WorthQueryApplicationObservedFact};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

/// Why a fact comparison gave no answer. The meter that pays for the
/// comparison stops separately, as a `CompanionPreflightStop`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQuerySourceCurrentnessFailure {
    /// The fact's own recorded bound refuses the comparison.
    WorkBudgetExceeded,
    Unavailable,
}

impl WorthQueryApplicationObservedFact {
    /// The most comparing this fact can cost: its probe, and for a native
    /// probe that found nothing the liveness check that answers it
    /// (`unanswered_probe`). A relation fact records no bound of its own, so
    /// only the meter bounds it.
    fn most_comparison_work(&self) -> usize {
        match self {
            Self::RetiredOutputEntity { .. }
            | Self::SourceEntity { .. }
            | Self::Entity { .. }
            | Self::WorkflowHistoryBasis { .. }
            | Self::WorkflowDefinitionPredecessor { lineage: None, .. } => 1,
            // A revision recorded with no comparison work cannot be compared.
            Self::SourceAdjacencyRevision {
                comparison_work_limit: 0,
                ..
            } => 0,
            Self::SourceFieldRevision { .. } | Self::SourceAdjacencyRevision { .. } => 2,
            Self::SourceAspectRevision { aspect, .. } => aspect.as_str().len().saturating_add(2),
            Self::Adjacency {
                maximum_work_units, ..
            }
            | Self::WorkflowDefinitionPredecessor {
                maximum_work_units, ..
            }
            | Self::WorkflowDefinitionCurrent {
                maximum_work_units, ..
            } => maximum_work_units.saturating_add(1),
            Self::WorkflowInstanceCapacity {
                maximum_instances, ..
            } => maximum_instances.saturating_mul(2).saturating_add(2),
            Self::IndexedEntitySelection {
                candidate_limit, ..
            } => candidate_limit.saturating_add(1),
            Self::Relation { .. } => usize::MAX,
            Self::Field { .. } | Self::AbsentField { .. } => 0,
        }
    }

    /// Whether the fact moved between the basis that read it and `snapshot`.
    /// The comparison reserves the least of its worst case and the meter's
    /// remaining work before it reads, reads within that reservation, and
    /// settles at what it spent. A comparison that cannot answer keeps its
    /// reservation charged and is its failure, never a movement. A read the
    /// reservation cut short is the meter's stop.
    pub(in crate::domain_computation::primary_graph) fn source_currentness_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        // Decision facts are value observations, not native revisions.
        // Output lineage must first rebase them at the committed snapshot.
        if matches!(self, Self::Field { .. } | Self::AbsentField { .. }) {
            return Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable));
        }
        let most = self.most_comparison_work();
        let reserved_work = most.min(admission.remaining_work());
        let reserved =
            admission.reserve_external_work(u64::try_from(reserved_work).unwrap_or(u64::MAX))?;
        match self.comparison(runtime, snapshot, reserved_work) {
            Ok((equal, work)) => {
                reserved.settle(u64::try_from(work).unwrap_or(u64::MAX))?;
                Ok(Ok(FactMovement::from_equal(equal)))
            }
            Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) if reserved_work < most => {
                // The reservation is the meter's remaining work, and the
                // comparison needed more.
                let maximum = admission.charged_work();
                Err(CompanionPreflightStop::WorkExhausted {
                    required: maximum.saturating_add(1),
                    maximum,
                })
            }
            Err(failure) => Ok(Err(failure)),
        }
    }

    fn comparison(
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
            Self::IndexedEntitySelection { .. } => {
                super::indexed_entity_selection::currentness(self, runtime, snapshot, maximum_work)
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

#[cfg(test)]
impl WorthQueryApplicationObservedFact {
    /// The comparison on a meter of `maximum_work` alone, with the work it
    /// charged. A meter it runs out is the comparison's work failure.
    pub(in crate::domain_computation::primary_graph) fn source_currentness_within(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        maximum_work: usize,
    ) -> Result<(FactMovement, usize), WorthQuerySourceCurrentnessFailure> {
        let mut admission = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: u64::try_from(maximum_work).unwrap_or(u64::MAX),
                maximum_preparation_bytes: 0,
            },
        );
        let movement = self
            .source_currentness_in(runtime, snapshot, &mut admission)
            .map_err(|_| WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)??;
        let work = usize::try_from(admission.charged_work()).unwrap_or(usize::MAX);
        Ok((movement, work))
    }
}
