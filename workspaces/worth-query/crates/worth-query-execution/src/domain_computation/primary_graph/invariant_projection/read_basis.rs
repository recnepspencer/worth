//! An operation reads one genuine Native phase. Applied rows do not borrow a
//! mutable runtime or acquire the graph owner's lock during Native preparation.
use super::*;
mod selection;
pub(super) use selection::ProjectionEntitySelection;
use worth_relational::facade::{
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalAppliedCandidateReadQualification,
        RelationalPinnedRead, RelationalRuntime, VisibilityProjectionView,
    },
    snapshots::SnapshotHandle,
};

pub enum NativeProjectionRead<'a> {
    Committed(&'a mut RelationalRuntime),
    Applied {
        before: &'a RelationalPinnedRead,
        candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    },
}

pub(super) struct ProjectionResolvedEntity {
    pub(super) entity: EntityId,
    pub(super) kind: KindId,
    pub(super) phase: Option<RelationalAppliedCandidateReadQualification>,
}

impl NativeProjectionRead<'_> {
    pub(super) fn origin<'a>(
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
    pub(super) fn candidate_identity(&self) -> Option<&RelationalAppliedCandidateReadIdentity> {
        match self {
            Self::Committed(_) => None,
            Self::Applied { candidate, .. } => Some(candidate.candidate_identity()),
        }
    }
    pub(super) fn runtime_instance_id(&self) -> u64 {
        match self {
            Self::Committed(runtime) => runtime.runtime_instance_id(),
            Self::Applied { candidate, .. } => candidate
                .candidate_identity()
                .proposal()
                .runtime_instance_id(),
        }
    }
    pub(super) const fn version(&self, snapshot: &SnapshotHandle) -> VersionId {
        match self {
            Self::Committed(_) => snapshot.version_id(),
            Self::Applied { candidate, .. } => candidate
                .candidate_identity()
                .proposal()
                .proposed_version_id(),
        }
    }
    pub(super) fn committed_runtime(&self) -> Option<&RelationalRuntime> {
        match self {
            Self::Committed(runtime) => Some(runtime),
            Self::Applied { .. } => None,
        }
    }
    pub(super) fn committed_runtime_mut(&mut self) -> Option<&mut RelationalRuntime> {
        match self {
            Self::Committed(runtime) => Some(runtime),
            Self::Applied { .. } => None,
        }
    }
    pub(super) fn pin(
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
    pub(super) fn projection(&self, snapshot: &SnapshotHandle) -> Option<VisibilityProjectionView> {
        match self {
            Self::Committed(runtime) => runtime.read_truth().project_snapshot(snapshot),
            Self::Applied { candidate, .. } => Some(candidate.project_candidate()),
        }
    }
    pub(super) fn observe_field<Output>(
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
        let Some(view) = self.projection(snapshot) else {
            return Ok(None);
        };
        super::super::application_attempt::observe_field_from_view_admitted(
            &view, execution, entity, kind, locator, action,
        )
    }
    pub(super) fn resolve_entity(
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
            Self::Applied { before, candidate } => {
                let (result, work) = installed.resolve_applied_with_work(
                    execution, before, candidate, entity, aspect, field, &expected,
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
