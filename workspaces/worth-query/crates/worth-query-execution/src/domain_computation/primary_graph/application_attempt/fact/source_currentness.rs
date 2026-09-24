use super::WorthQueryApplicationObservedFact;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQuerySourceCurrentnessFailure {
    WorkBudgetExceeded,
    Unavailable,
}

impl WorthQueryApplicationObservedFact {
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
            Self::SourceEntity { entity_id } => Ok((
                runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .is_some_and(|view| {
                        view.entity_record_with_projection_scope(
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
                1,
            )),
            Self::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => Ok((
                runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .and_then(|view| view.entity_aspect_version(*entity_id, aspect))
                    == Some(*native_revision),
                1,
            )),
            Self::SourceFieldRevision { entity_id, locator, native_revision } => Ok((
                native_revision.is_some_and(|expected| {
                    runtime.read_truth().project_snapshot(snapshot)
                        .and_then(|view| view.entity_field_revision(*entity_id, locator)) == Some(expected)
                }),
                1,
            )),
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
                Ok((comparison.revision() == *native_revision, comparison.work_units()))
            }
            Self::Entity { .. } => Ok((self.remains_equal_in(runtime, snapshot), 1)),
            // Decision facts are value observations, not native revisions.
            // Output lineage must first rebase them at the committed snapshot.
            Self::Field { .. } | Self::AbsentField { .. } => {
                Err(WorthQuerySourceCurrentnessFailure::Unavailable)
            }
            _ => Err(WorthQuerySourceCurrentnessFailure::Unavailable),
        }
    }
}
