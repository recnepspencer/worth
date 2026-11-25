//! Phase-bound Home readmission metadata projection over a recovered branch.
use crate::durability::HomeReadmissionMetadataAuthority;
use crate::identity::data::KindId;
use crate::visibility::materialization::read_records::reader::{
    BoundedEntityKindTruthRead, BoundedRelationKindTruthRead, EntityKindTruthReadLimitExceeded,
    RelationKindTruthReadDenial,
};

use super::VisibilityProjectionView;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;
use worth_relational_branch::internal_port::mvcc::observation::RelationalBranchObservationOwnerPort as _;
use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::creation::VisibilityProjectionViewOwnerPort as _;

/// Typed denial for scans through [`HomeReadmissionMetadataView`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeReadmissionMetadataReadDenial {
    /// The active Home readmission phase ended (authority dropped,
    /// `finish_home_readmission` ran, or a post-recovery commit landed).
    PhaseClosed,
    /// The requested kind was not sealed as an owner-admitted metadata kind at
    /// initial schema installation (e.g., an application kind).
    KindNotAdmitted(KindId),
    /// Underlying execution denial (budget, cancellation, allocation, etc.).
    Execution(crate::execution::RelationalExecutionDenialCause),
}

/// Phase-bound, non-cloneable projection for Home readmission metadata on a
/// recovered branch whose generated records may be suspended.
///
/// Constructed exclusively via [`VisibilityReadContext::project_home_readmission_metadata`]
/// borrowing an active linear [`HomeReadmissionMetadataAuthority`] issued by
/// [`crate::durability::RecoveredRelationalRuntimeAuthority::begin_home_readmission_metadata`].
///
/// Ordinary callers cannot invoke arbitrary exact-entity reads:
/// ```compile_fail,E0599
/// use worth_relational::facade::identity::EntityId;
/// use worth_relational::facade::runtime::HomeReadmissionMetadataView;
/// fn cannot_read_arbitrary_entity(
///     view: &HomeReadmissionMetadataView<'_>,
///     execution: &worth_execution::ExecutionRequest<'_, '_>,
///     entity: EntityId,
/// ) {
///     let _ = view.read_exact_entity_state(execution, entity, |_, _| ());
/// }
/// ```
///
/// Views cannot be cloned:
/// ```compile_fail,E0277
/// use worth_relational::facade::runtime::HomeReadmissionMetadataView;
/// fn require_clone<T: Clone>() {}
/// require_clone::<HomeReadmissionMetadataView<'_>>();
/// ```
///
/// And a view cannot outlive its [`HomeReadmissionMetadataAuthority`]:
/// ```compile_fail,E0597
/// use worth_relational::facade::durability::HomeReadmissionMetadataAuthority;
/// use worth_relational::facade::mvcc::RelationalBranchObservation;
/// use worth_relational::facade::runtime::{HomeReadmissionMetadataView, RelationalRuntime};
/// fn cannot_outlive_authority<'r>(
///     runtime: &'r RelationalRuntime,
///     authority: HomeReadmissionMetadataAuthority,
///     observation: &RelationalBranchObservation,
/// ) -> HomeReadmissionMetadataView<'static> {
///     runtime
///         .read_truth()
///         .project_home_readmission_metadata(&authority, observation)
///         .unwrap()
/// }
/// ```
pub struct HomeReadmissionMetadataView<'authority> {
    inner: VisibilityProjectionView,
    authority: &'authority HomeReadmissionMetadataAuthority,
}

impl std::fmt::Debug for HomeReadmissionMetadataView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HomeReadmissionMetadataView")
            .field("active", &self.authority.is_active())
            .finish_non_exhaustive()
    }
}

impl HomeReadmissionMetadataView<'_> {
    /// Scan live entities of one owner-admitted metadata kind during the active
    /// Home readmission phase.
    pub fn bounded_entities_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedEntityKindTruthRead, EntityKindTruthReadLimitExceeded>,
        HomeReadmissionMetadataReadDenial,
    > {
        if !self.authority.is_active() {
            return Err(HomeReadmissionMetadataReadDenial::PhaseClosed);
        }
        if !self.authority.allows_entity_kind(kind_id) {
            return Err(HomeReadmissionMetadataReadDenial::KindNotAdmitted(kind_id));
        }
        self.inner
            .bounded_entities_of_kind(execution, kind_id, maximum_work_units)
            .map_err(HomeReadmissionMetadataReadDenial::Execution)
    }

    /// Scan live relations of one owner-admitted metadata kind during the active
    /// Home readmission phase.
    pub fn bounded_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedRelationKindTruthRead, RelationKindTruthReadDenial>,
        HomeReadmissionMetadataReadDenial,
    > {
        if !self.authority.is_active() {
            return Err(HomeReadmissionMetadataReadDenial::PhaseClosed);
        }
        if !self.authority.allows_relation_kind(kind_id) {
            return Err(HomeReadmissionMetadataReadDenial::KindNotAdmitted(kind_id));
        }
        self.inner
            .bounded_relations_of_kind(execution, kind_id, maximum_work_units)
            .map_err(HomeReadmissionMetadataReadDenial::Execution)
    }
}

impl<'runtime> VisibilityReadContext<'runtime> {
    /// Project owner-admitted internal metadata records during the active Home
    /// readmission phase using linear authority from
    /// [`crate::durability::RecoveredRelationalRuntimeAuthority::begin_home_readmission_metadata`].
    ///
    /// Refuses once the readmission phase closes (including on service handoff
    /// or any post-recovery commit), refuses foreign runtime authorities, and
    /// refuses post-recovery observations beyond the recovered commit frontier.
    pub fn project_home_readmission_metadata<'authority>(
        &self,
        authority: &'authority HomeReadmissionMetadataAuthority,
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Result<HomeReadmissionMetadataView<'authority>, crate::branch::basis_denial::RelationalBranchBasisDenial>
    {
        if !authority.is_active() {
            return Err(crate::branch::basis_denial::RelationalBranchBasisDenial::OwnerUnavailable);
        }
        let expected = self.runtime().runtime_instance_id();
        if authority.runtime_instance_id() != expected {
            return Err(crate::branch::basis_denial::RelationalBranchBasisDenial::ForeignRuntime {
                expected_runtime_instance_id: expected,
                actual_runtime_instance_id: authority.runtime_instance_id(),
            });
        }
        if observation.identity().runtime_instance_id() != expected {
            return Err(crate::branch::basis_denial::RelationalBranchBasisDenial::ForeignRuntime {
                expected_runtime_instance_id: expected,
                actual_runtime_instance_id: observation.identity().runtime_instance_id(),
            });
        }
        if let Some(commit_id) = observation.selected_root().commit_id()
            && authority
                .recovered_latest_commit()
                .is_none_or(|latest| commit_id > latest)
        {
            return Err(crate::branch::basis_denial::RelationalBranchBasisDenial::OwnerUnavailable);
        }
        let basis = crate::history::retention::VisibilitySnapshotBasis::from_observation(observation);
        Ok(HomeReadmissionMetadataView {
            inner: VisibilityProjectionView::new(
                self.runtime(),
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis),
            ),
            authority,
        })
    }
}
