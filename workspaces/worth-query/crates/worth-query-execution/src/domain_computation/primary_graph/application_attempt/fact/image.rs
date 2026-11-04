//! Physical facts read one committed image or one actual applied Native pin.
use super::super::{observation, WorthQueryApplicationAttemptDenial as Denial};
use super::{WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationObservedRelation};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, AspectKey};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime,
        VisibilityProjectionView,
    },
    snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause,
};

pub(super) struct FactReadImage<'a> {
    pub(super) runtime: &'a RelationalRuntime,
    pub(super) before: &'a SnapshotHandle,
    pub(super) applied: Option<&'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>>,
}
impl FactReadImage<'_> {
    pub(super) fn view(&self) -> Option<VisibilityProjectionView> {
        match self.applied {
            Some(pin) => Some(pin.project_candidate()),
            None => self.runtime.read_truth().project_snapshot(self.before),
        }
    }
    pub(super) fn field(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
    ) -> Result<
        Option<observation::WorthQueryApplicationFieldObservation>,
        RelationalExecutionDenialCause,
    > {
        match self.applied {
            Some(pin) => observation::observe_field_from_view_admitted(
                &pin.project_candidate(),
                execution,
                entity,
                kind,
                locator,
                |raw| observation::WorthQueryApplicationFieldObservation::from(raw),
            ),
            None => observation::observe_field(
                self.runtime,
                execution,
                self.before,
                entity,
                kind,
                locator,
            ),
        }
    }
    pub(super) fn aspect_presence(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
        kind: KindId,
        aspect: &AspectKey,
    ) -> Result<Option<bool>, RelationalExecutionDenialCause> {
        match self.applied {
            Some(pin) => observation::observe_aspect_presence_in_view(
                &pin.project_candidate(),
                execution,
                entity,
                kind,
                aspect,
            ),
            None => observation::observe_aspect_presence(
                self.runtime,
                execution,
                self.before,
                entity,
                kind,
                aspect,
            ),
        }
    }
    pub(super) fn live_kind(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<Option<Option<KindId>>, RelationalExecutionDenialCause> {
        match self.applied {
            Some(pin) => {
                let view = pin.project_candidate();
                let live = view.entity_record_with_projection_scope(
                    execution,
                    entity,
                    worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                    |record| {
                        Some(
                            (record.lifecycle()
                                == worth_relational::facade::storage::RecordLifecycleState::Live)
                                .then(|| record.kind_id()),
                        )
                    },
                )?;
                if live.is_some() {
                    return Ok(live);
                }
                Ok(view
                    .read_entity_retirement(execution, entity)?
                    .map(|_| None))
            }
            None => self
                .runtime
                .read_truth()
                .read_exact_snapshot_live_entity_kind_status(execution, self.before, entity),
        }
    }
    pub(super) fn aspect_revision(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
        aspect: &AspectKey,
    ) -> Result<Option<Option<u64>>, Denial> {
        match self.applied {
            Some(pin) => pin.project_candidate().read_entity_aspect_version(execution, entity, aspect)
                .map_err(|cause| match cause {
                    worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => Denial::relational_read_stopped(cause, "source aspect comparison"),
                    _ => Denial::new(super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact, "source aspect comparison"),
                }),
            None => self.runtime.read_truth().read_exact_snapshot_entity_aspect_version(execution, self.before, entity, aspect)
                .map_err(|cause| Denial::relational_read_stopped(cause, "source aspect comparison")),
        }
    }
    pub(super) fn relations(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        kind: KindId,
        from: EntityId,
        to: EntityId,
    ) -> Result<Vec<worth_relational::facade::identity::RelationId>, Denial> {
        match self.applied {
            Some(pin) => Ok(super::observe_adjacency_in_pin(
                pin,
                execution,
                kind,
                from,
                WorthQueryApplicationAdjacencyDirection::Outgoing,
                None,
            )?
            .unwrap_or_default()
            .into_iter()
            .filter(|edge| edge.to == to)
            .map(|edge| edge.relation_id)
            .collect()),
            None => {
                observation::exact_relations(self.runtime, execution, self.before, kind, from, to)
            }
        }
    }
    pub(super) fn adjacency(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        kind: KindId,
        anchor: EntityId,
        direction: WorthQueryApplicationAdjacencyDirection,
        limit: usize,
    ) -> Result<Option<Vec<WorthQueryApplicationObservedRelation>>, Denial> {
        match self.applied {
            Some(pin) => super::observe_adjacency_in_pin(
                pin,
                execution,
                kind,
                anchor,
                direction,
                Some(limit),
            ),
            None => super::adjacency::observe_adjacency(
                self.runtime,
                execution,
                self.before,
                kind,
                anchor,
                direction,
                limit,
            ),
        }
    }
}
