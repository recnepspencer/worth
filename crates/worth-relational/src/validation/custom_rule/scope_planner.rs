use crate::identity::data::VersionId;
use crate::transactions::data::MergedCommitPlan;
use crate::validation::engine::state_view::InvariantStateView;
use crate::validation::engine::InvariantRuntimeView;
use crate::validation::engine::{InvariantObservation, InvariantObservationKind};

use super::structural_views::{StructuralAspectStateView, StructuralRelationView};
use super::touched_scope_collection::collect_touched_structural_set;
use super::traversal::BoundedStructuralTraversal;
use crate::validation::data::{StructuralCountView, TouchedStructuralSet};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct PreparedCustomInvariantScope {
    touched: Arc<TouchedStructuralSet>,
}

impl PreparedCustomInvariantScope {
    pub(crate) fn capture(
        observation: &InvariantObservation<'_>,
        version_id: VersionId,
        merged_plan: Option<&MergedCommitPlan>,
        work: &super::CustomInvariantWorkMeter,
    ) -> Self {
        let state_view = InvariantStateView::new(
            observation.enforcement_partition_access(),
            observation.enforcement_version_id(version_id),
        );
        Self {
            touched: Arc::new(collect_touched_structural_set(
                &state_view,
                merged_plan,
                work,
            )),
        }
    }

    pub(crate) fn retain_restricted(
        &self,
        state: &InvariantStateView<'_>,
        committed: &InvariantStateView<'_>,
        access: &crate::validation::data::CustomInvariantAccessContract,
        work: &super::CustomInvariantWorkMeter,
    ) -> Arc<TouchedStructuralSet> {
        let count = [
            self.touched.visible_entity_ids().len(),
            self.touched.visible_relation_ids().len(),
            self.touched.touched_partitions().len(),
            self.touched.planned_entity_deletes().len(),
            self.touched.planned_entity_creates().len(),
            self.touched.planned_relation_creates().len(),
            self.touched.planned_relation_deletes().len(),
            self.touched.planned_relation_endpoint_updates().len(),
        ]
        .into_iter()
        .fold(0usize, usize::saturating_add);
        if !work.try_charge(count) {
            return Arc::new(TouchedStructuralSet::new(
                Arc::from([]),
                Arc::from([]),
                Arc::from([]),
                Arc::from([]),
                Arc::from([]),
                Arc::from([]),
                Arc::from([]),
                Arc::from([]),
            ));
        }
        Arc::new(TouchedStructuralSet::new(
            self.touched
                .visible_entity_ids()
                .iter()
                .copied()
                .filter(|id| {
                    state
                        .entity_metadata(*id)
                        .or_else(|| {
                            work.try_charge(1)
                                .then(|| committed.entity_metadata(*id))
                                .flatten()
                        })
                        .is_some_and(|metadata| access.affects_entity(metadata.kind_id))
                })
                .collect::<Vec<_>>()
                .into(),
            self.touched
                .visible_relation_ids()
                .iter()
                .copied()
                .filter(|id| {
                    state
                        .relation_metadata(*id)
                        .or_else(|| {
                            work.try_charge(1)
                                .then(|| committed.relation_metadata(*id))
                                .flatten()
                        })
                        .is_some_and(|metadata| access.affects_relation(metadata.kind_id))
                })
                .collect::<Vec<_>>()
                .into(),
            self.touched.touched_partitions().to_vec().into(),
            self.touched
                .planned_entity_deletes()
                .iter()
                .copied()
                .filter(|id| {
                    state
                        .entity_metadata(*id)
                        .or_else(|| {
                            work.try_charge(1)
                                .then(|| committed.entity_metadata(*id))
                                .flatten()
                        })
                        .is_some_and(|metadata| access.affects_entity(metadata.kind_id))
                })
                .collect::<Vec<_>>()
                .into(),
            self.touched
                .planned_entity_creates()
                .iter()
                .filter(|create| access.affects_entity(create.kind_id()))
                .cloned()
                .collect::<Vec<_>>()
                .into(),
            self.touched
                .planned_relation_creates()
                .iter()
                .filter(|create| access.affects_relation(create.kind_id()))
                .cloned()
                .collect::<Vec<_>>()
                .into(),
            self.touched
                .planned_relation_deletes()
                .iter()
                .copied()
                .filter(|id| {
                    state
                        .relation_metadata(*id)
                        .or_else(|| {
                            work.try_charge(1)
                                .then(|| committed.relation_metadata(*id))
                                .flatten()
                        })
                        .is_some_and(|metadata| access.affects_relation(metadata.kind_id))
                })
                .collect::<Vec<_>>()
                .into(),
            self.touched
                .planned_relation_endpoint_updates()
                .iter()
                .filter(|update| access.affects_relation(update.kind_id()))
                .cloned()
                .collect::<Vec<_>>()
                .into(),
        ))
    }
}

pub struct CustomInvariantScopePlanner<'runtime> {
    observation_kind: InvariantObservationKind,
    version_id: VersionId,
    current_version_id: VersionId,
    touched: Arc<TouchedStructuralSet>,
    aspect_states: StructuralAspectStateView<'runtime>,
    committed_aspect_states: StructuralAspectStateView<'runtime>,
    relations: StructuralRelationView<'runtime>,
    counts: StructuralCountView,
    traversal: BoundedStructuralTraversal<'runtime>,
    work: super::CustomInvariantWorkMeter,
}

impl<'runtime> CustomInvariantScopePlanner<'runtime> {
    pub(crate) fn new_at_current_version(
        runtime: &InvariantRuntimeView<'runtime>,
        observation: &'runtime InvariantObservation<'runtime>,
        version_id: VersionId,
        current_version_id: VersionId,
        prepared_scope: &PreparedCustomInvariantScope,
        work: super::CustomInvariantWorkMeter,
        access: std::sync::Arc<crate::validation::data::CustomInvariantAccessContract>,
    ) -> Self {
        let state_view = InvariantStateView::new(
            observation.enforcement_partition_access(),
            observation.enforcement_version_id(version_id),
        );
        let committed_state_view =
            InvariantStateView::new(observation.committed_partition_access(), current_version_id);
        work.try_charge(1);
        let touched =
            prepared_scope.retain_restricted(&state_view, &committed_state_view, &access, &work);
        let aspect_states =
            StructuralAspectStateView::new(state_view, work.clone(), access.clone());
        let relations = StructuralRelationView::new(state_view, work.clone(), access.clone());
        let counts = StructuralCountView::from_touched_scope(&touched);
        let traversal = BoundedStructuralTraversal::new(
            runtime.performance_access(),
            relations.clone(),
            &touched,
            work.clone(),
        );
        Self {
            observation_kind: observation.kind(),
            version_id,
            current_version_id,
            touched,
            aspect_states,
            committed_aspect_states: StructuralAspectStateView::new(
                committed_state_view,
                work.clone(),
                access,
            ),
            relations,
            counts,
            traversal,
            work,
        }
    }

    pub fn observation_kind(&self) -> InvariantObservationKind {
        self.observation_kind
    }

    pub fn version_id(&self) -> VersionId {
        self.version_id
    }

    pub fn current_version_id(&self) -> VersionId {
        self.current_version_id
    }

    pub fn touched(&self) -> &TouchedStructuralSet {
        &self.touched
    }

    pub fn aspect_states(&self) -> StructuralAspectStateView<'runtime> {
        self.aspect_states.clone()
    }

    /// Read the immutable committed basis that the proposed view is checked against.
    pub fn committed_aspect_states(&self) -> StructuralAspectStateView<'runtime> {
        self.committed_aspect_states.clone()
    }

    pub fn relations(&self) -> StructuralRelationView<'runtime> {
        self.relations.clone()
    }

    pub fn counts(&self) -> StructuralCountView {
        self.counts
    }

    pub fn traversal(&self) -> &BoundedStructuralTraversal<'runtime> {
        &self.traversal
    }

    pub(crate) fn work_meter(&self) -> super::CustomInvariantWorkMeter {
        self.work.clone()
    }
}
