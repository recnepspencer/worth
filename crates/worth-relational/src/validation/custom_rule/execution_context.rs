use crate::identity::data::VersionId;
use crate::validation::engine::state_view::InvariantStateView;
use crate::validation::engine::InvariantRuntimeView;
use crate::validation::engine::{InvariantObservation, InvariantObservationKind};

use super::structural_views::{StructuralAspectStateView, StructuralRelationView};
use super::traversal::{BoundedStructuralTraversal, CustomInvariantTraversalSummary};
use crate::validation::data::{
    CustomInvariantTouchedSummary, StructuralCountView, TouchedStructuralSet,
};
use std::sync::Arc;

pub struct CustomInvariantExecutionContext<'runtime> {
    performance: crate::performance::PerformanceAccess<'runtime>,
    observation_kind: InvariantObservationKind,
    version_id: VersionId,
    current_version_id: VersionId,
    touched: Arc<TouchedStructuralSet>,
    aspect_states: StructuralAspectStateView<'runtime>,
    committed_aspect_states: StructuralAspectStateView<'runtime>,
    relations: StructuralRelationView<'runtime>,
    counts: StructuralCountView,
    traversal: BoundedStructuralTraversal<'runtime>,
    proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    work: super::CustomInvariantWorkMeter,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CustomInvariantProvenance {
    pub observation_kind: InvariantObservationKind,
    pub version_id: VersionId,
    pub current_version_id: VersionId,
    pub touched: CustomInvariantTouchedSummary,
    pub counts: StructuralCountView,
    pub traversal: CustomInvariantTraversalSummary,
    #[serde(skip)]
    pub proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    pub work_units: std::num::NonZeroU64,
}

impl<'runtime> CustomInvariantExecutionContext<'runtime> {
    pub(crate) fn new(
        runtime: &InvariantRuntimeView<'runtime>,
        observation: &'runtime InvariantObservation<'runtime>,
        version_id: VersionId,
        current_version_id: VersionId,
        prepared_scope: &super::scope_planner::PreparedCustomInvariantScope,
        work: super::CustomInvariantWorkMeter,
        access: std::sync::Arc<crate::validation::data::CustomInvariantAccessContract>,
    ) -> Self {
        let version_id = observation.enforcement_version_id(version_id);
        let state_view =
            InvariantStateView::new(observation.enforcement_partition_access(), version_id);
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
        let proposal_identity = observation.proposal_identity().cloned();
        Self {
            performance: runtime.performance_access(),
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
            proposal_identity,
            work,
        }
    }

    pub fn observation_kind(&self) -> InvariantObservationKind {
        self.observation_kind
    }

    pub(crate) fn performance_access(&self) -> &crate::performance::PerformanceAccess<'runtime> {
        &self.performance
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

    pub fn provenance(&self) -> CustomInvariantProvenance {
        CustomInvariantProvenance {
            observation_kind: self.observation_kind,
            version_id: self.version_id,
            current_version_id: self.current_version_id,
            touched: self.touched.provenance_summary(),
            counts: self.counts,
            traversal: self.traversal.summary(),
            proposal_identity: self.proposal_identity.clone(),
            work_units: self.work.consumed(),
        }
    }
}
