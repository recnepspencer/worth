//! An operation reads one genuine Native phase. Applied rows do not borrow a
//! mutable runtime or acquire the graph owner's lock during Native preparation.
use super::*;
pub mod selection;
pub use selection::ProjectionEntitySelection;
pub mod own_outputs;
pub use own_outputs::OwnOutputOverlay;
use worth_relational::facade::{
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalAppliedCandidateReadQualification,
        RelationalPinnedRead, RelationalRuntime, VisibilityProjectionView,
    },
    snapshots::SnapshotHandle,
};

pub enum NativeProjectionRead<'a> {
    Committed(&'a RelationalRuntime),
    Applied {
        before: &'a RelationalPinnedRead,
        candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        /// A successor round's own records, read before its chain (§1.5).
        own_outputs: Option<&'a OwnOutputOverlay>,
    },
}

pub struct ProjectionResolvedEntity {
    pub entity: EntityId,
    pub kind: KindId,
    pub phase: Option<RelationalAppliedCandidateReadQualification>,
}

impl<'r> NativeProjectionRead<'r> {
    pub fn origin<'a>(
        &self,
        snapshot: &'a SnapshotHandle,
    ) -> super::super::projection_read_origin::ProjectionReadOriginRef<'a> {
        match self {
            Self::Committed(_) => {
                super::super::projection_read_origin::ProjectionReadOriginRef::Committed(snapshot)
            }
            Self::Applied { candidate, .. } => {
                super::super::projection_read_origin::ProjectionReadOriginRef::Applied(
                    candidate.candidate_identity().qualification(),
                )
            }
        }
    }
    pub fn candidate_identity(&self) -> Option<&RelationalAppliedCandidateReadIdentity> {
        match self {
            Self::Committed(_) => None,
            Self::Applied { candidate, .. } => Some(candidate.candidate_identity()),
        }
    }
    pub fn runtime_instance_id(&self) -> u64 {
        match self {
            Self::Committed(runtime) => runtime.runtime_instance_id(),
            Self::Applied { candidate, .. } => candidate
                .candidate_identity()
                .proposal()
                .runtime_instance_id(),
        }
    }
    pub const fn version(&self, snapshot: &SnapshotHandle) -> VersionId {
        match self {
            Self::Committed(_) => snapshot.version_id(),
            Self::Applied { candidate, .. } => candidate
                .candidate_identity()
                .proposal()
                .proposed_version_id(),
        }
    }
    /// The applied candidate this read borrows, for the read's whole life.
    pub fn applied_pin_lent(
        &self,
    ) -> Option<&'r RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>> {
        match self {
            Self::Applied { candidate, .. } => Some(candidate),
            Self::Committed(_) => None,
        }
    }
    pub fn applied_pin(
        &self,
    ) -> Option<&RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>> {
        match self {
            Self::Applied { candidate, .. } => Some(candidate),
            Self::Committed(_) => None,
        }
    }
    /// A successor round's own-output view, when this read serves one.
    pub fn own_outputs(&self) -> Option<&'r OwnOutputOverlay> {
        match self {
            Self::Applied { own_outputs, .. } => *own_outputs,
            Self::Committed(_) => None,
        }
    }
    /// The committed runtime this read borrows, for the read's whole life:
    /// a nested committed read lends it without holding the reader.
    pub fn committed_runtime_lent(&self) -> Option<&'r RelationalRuntime> {
        match self {
            Self::Committed(runtime) => Some(runtime),
            Self::Applied { .. } => None,
        }
    }
    pub fn committed_runtime(&self) -> Option<&RelationalRuntime> {
        match self {
            Self::Committed(runtime) => Some(runtime),
            Self::Applied { .. } => None,
        }
    }
    pub fn pin(
        &self,
        snapshot: &SnapshotHandle,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        RelationalPinnedRead<()>,
        worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial,
    > {
        match self {
            Self::Committed(runtime) => runtime
                .read_truth()
                .pin_read_capability(snapshot, &[], execution)
                .map(RelationalPinnedRead::into_projection_read),
            Self::Applied { candidate, .. } => Ok((*candidate).clone().into_projection_read()),
        }
    }
    pub fn projection(&self, snapshot: &SnapshotHandle) -> Option<VisibilityProjectionView> {
        match self {
            Self::Committed(runtime) => runtime.read_truth().project_snapshot(snapshot),
            Self::Applied { candidate, .. } => Some(candidate.project_candidate()),
        }
    }
    /// The projection one record is read at: for a successor round, a record
    /// its own chain wrote is read as it was at the chain head (§1.5).
    pub fn projection_of(
        &self,
        snapshot: &SnapshotHandle,
        entity: EntityId,
    ) -> Option<VisibilityProjectionView> {
        let own = match self {
            Self::Applied {
                own_outputs: Some(own),
                ..
            } => own.head_for(entity).map(|head| head.project_candidate()),
            _ => None,
        };
        own.or_else(|| self.projection(snapshot))
    }
    /// The projection one relation is read at: for a successor round, a
    /// relation its own chain linked or unlinked is read as it was at the
    /// chain head (#277), as its adjacency is served.
    pub fn projection_of_relation(
        &self,
        snapshot: &SnapshotHandle,
        relation: worth_relational::facade::identity::RelationId,
    ) -> Option<VisibilityProjectionView> {
        let own = match self {
            Self::Applied {
                own_outputs: Some(own),
                ..
            } if own.relations().covers(relation) => Some(own.head().project_candidate()),
            _ => None,
        };
        own.or_else(|| self.projection(snapshot))
    }
    pub fn observe_field<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        entity: EntityId,
        kind: KindId,
        locator: &worth_foundational::facade::AspectFieldLocator,
        action: impl for<'raw> FnOnce(
            worth_relational::facade::runtime::RelationalProjectedFieldObservation<'raw>,
        ) -> Output,
    ) -> Result<
        Option<Output>,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    > {
        let Some(view) = self.projection_of(snapshot, entity) else {
            return Ok(None);
        };
        let observed = super::super::application_attempt::observe_field_from_view_admitted(
            &view, execution, entity, kind, locator, action,
        )?;
        // Served at the head pre-pin and absent there: the chain created it.
        if observed.is_none()
            && let Self::Applied {
                own_outputs: Some(own),
                ..
            } = self
        {
            own.mark_incoherent(entity);
        }
        Ok(observed)
    }
    pub fn resolve_entity(
        &self,
        installed: &super::super::WorthQueryInstalledEntityResolutionContext,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: worth_foundational::facade::AspectValue,
    ) -> (
        Result<ProjectionResolvedEntity, super::super::WorthQueryEntityResolutionDenial>,
        usize,
    ) {
        match self {
            Self::Committed(runtime) => {
                let truth = match installed.at_snapshot(
                    runtime,
                    snapshot,
                    super::super::WorthQueryPrincipalResolutionMode::Ordinary,
                ) {
                    Ok(truth) => truth,
                    Err(cause) => return (Err(cause), 0),
                };
                let (result, work) =
                    truth.resolve_with_work(execution, entity, aspect, field, expected);
                (
                    result.map(|value| ProjectionResolvedEntity {
                        entity: value.entity_id(),
                        kind: value.entity_kind(),
                        phase: None,
                    }),
                    work,
                )
            }
            Self::Applied {
                before,
                candidate,
                own_outputs,
            } => {
                // A record the successor's own chain wrote resolves as it was
                // before the chain: a Birth's own output is absent again, and
                // a key the chain moved away resolves to its record again.
                // Every other record resolves as it is now.
                let covers = |entity| own_outputs.is_some_and(|own| own.covers(entity));
                let (result, work) = installed.resolve_applied_merged_with_work(
                    execution,
                    before,
                    candidate,
                    own_outputs.map(|own| (own.head(), &covers as &dyn Fn(_) -> bool)),
                    entity,
                    aspect,
                    field,
                    &expected,
                );
                (
                    result.map(|value| ProjectionResolvedEntity {
                        entity: value.entity,
                        kind: value.kind,
                        phase: Some(value.candidate),
                    }),
                    work,
                )
            }
        }
    }
}

mod court_doors_for_test;
