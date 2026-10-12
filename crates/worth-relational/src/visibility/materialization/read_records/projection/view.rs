use crate::identity::data::{KindId, VersionId};
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;

use super::super::reader::VisibilityReadContext;
use super::contracts::{assert_declared_projection_aspects, ProjectionAspectScope};
use super::projection_records::{EntityRecordProjection, RelationRecordProjection};

#[derive(Debug, Clone)]
pub struct VisibilityProjectionView {
    pub(super) runtime_identity: u64,
    pub(super) current_version: VersionId,
    pub(super) instrumentation: crate::runtime::RuntimeInstrumentation,
    pub(super) symbols: crate::symbols::data::StringInterner,
    pub(super) schema: super::schema_custody::ProjectionSchemaCustody,
    pub(super) basis: crate::visibility::snapshot_states::SnapshotStateBasis,
}

impl VisibilityProjectionView {
    pub const fn version_id(&self) -> VersionId {
        self.basis.version_id()
    }

    pub(crate) fn selected_root(
        &self,
    ) -> Option<&std::sync::Arc<crate::branch::RelationalBranchRoot>> {
        self.basis.root()
    }

    pub(crate) fn is_exact_basis(&self) -> bool {
        matches!(
            self.basis,
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(_)
        )
    }

    pub(crate) fn is_from_runtime(&self, runtime: &RelationalRuntime) -> bool {
        self.runtime_identity == runtime.runtime_instance_id()
    }

    pub(crate) fn selected_schema_authority(
        &self,
    ) -> Option<&crate::branch::RelationalBranchRootSchemaAuthority> {
        self.basis.root().map(|root| root.schema_authority())
    }

    pub(super) fn reader(&self) -> super::super::reader::ProjectionRecordReader<'_> {
        super::super::reader::ProjectionRecordReader::new(
            self.current_version,
            &self.instrumentation,
        )
    }

    pub(super) fn assert_entity_projection_contract<T: EntityRecordProjection>(
        &self,
    ) -> ProjectionAspectScope {
        let projection_scope = T::projection_scope();
        assert_declared_projection_aspects(
            &projection_scope,
            self.entity_aspect_plan(T::KIND),
            "entity",
            T::KIND,
        );
        projection_scope
    }

    pub(super) fn assert_relation_projection_contract<T: RelationRecordProjection>(
        &self,
    ) -> ProjectionAspectScope {
        let projection_scope = T::projection_scope();
        assert_declared_projection_aspects(
            &projection_scope,
            self.relation_aspect_plan(T::KIND),
            "relation",
            T::KIND,
        );
        projection_scope
    }

    pub(super) fn assert_entity_projection_scope(
        &self,
        kind_id: KindId,
        projection_scope: &ProjectionAspectScope,
    ) {
        assert_declared_projection_aspects(
            projection_scope,
            self.entity_aspect_plan(kind_id),
            "entity",
            kind_id,
        );
    }

    pub(super) fn assert_relation_projection_scope(
        &self,
        kind_id: KindId,
        projection_scope: &ProjectionAspectScope,
    ) {
        assert_declared_projection_aspects(
            projection_scope,
            self.relation_aspect_plan(kind_id),
            "relation",
            kind_id,
        );
    }

    pub(super) fn entity_aspect_plan(
        &self,
        kind_id: KindId,
    ) -> Option<&crate::schema::data::LoweredAspectContractPlan> {
        match &self.basis {
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                basis.root().schema_authority().entity_aspect_plan(kind_id)
            }
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(_) => self
                .basis
                .root()
                .and_then(|root| root.schema_authority().entity_aspect_plan(kind_id)),
        }
    }

    pub(super) fn relation_aspect_plan(
        &self,
        kind_id: KindId,
    ) -> Option<&crate::schema::data::LoweredAspectContractPlan> {
        match &self.basis {
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => basis
                .root()
                .schema_authority()
                .relation_aspect_plan(kind_id),
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(_) => self
                .basis
                .root()
                .and_then(|root| root.schema_authority().relation_aspect_plan(kind_id)),
        }
    }
}

impl<'runtime> VisibilityReadContext<'runtime> {
    pub(crate) fn project_branch_head(
        &self,
        branch_id: &crate::history::data::BranchId,
        version_id: VersionId,
    ) -> Result<Option<VisibilityProjectionView>, crate::branch::RelationalBranchBasisDenial> {
        let basis = crate::visibility::snapshot_states::VisibilitySnapshotBasis::capture_current(
            self.runtime(),
            branch_id,
            version_id,
        )?;
        Ok(basis.map(|basis| {
            VisibilityProjectionView::new(
                self.runtime(),
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis),
            )
        }))
    }

    pub(crate) fn project_historical_version(
        &self,
        version_id: VersionId,
    ) -> VisibilityProjectionView {
        self.try_project_historical_version(version_id)
            .expect("internal historical projection requires retained MVCC coverage")
    }

    /// Projects the exact state at `version_id` on the branch that committed
    /// it, so a fork's commit is read from the fork's own root.
    pub fn try_project_historical_version(
        &self,
        version_id: VersionId,
    ) -> Result<VisibilityProjectionView, crate::branch::RelationalBranchBasisDenial> {
        let basis = crate::visibility::cache_state::historical_basis_for_retained_version(
            self.runtime(),
            version_id,
        )
        .map_err(historical_projection_denial)?;
        Ok(VisibilityProjectionView::new(
            self.runtime(),
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis),
        ))
    }

    pub(crate) fn try_project_retained_commit(
        &self,
        commit_id: crate::history::data::CommitId,
        branch_id: crate::history::data::BranchId,
        version_id: VersionId,
    ) -> Result<VisibilityProjectionView, crate::branch::RelationalBranchBasisDenial> {
        let basis =
            crate::visibility::snapshot_states::HistoricalVisibilityBasis::resolve_retained_commit(
                self.runtime(),
                commit_id,
                branch_id,
                version_id,
            )
            .map_err(historical_projection_denial)?;
        Ok(VisibilityProjectionView::new(
            self.runtime(),
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis),
        ))
    }

    pub fn project_snapshot(
        &self,
        handle: &SnapshotHandle,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Option<VisibilityProjectionView> {
        let basis =
            crate::visibility::snapshot_states::resolve_snapshot_basis(self.runtime(), handle)?;
        if basis.root().has_materialization_unavailable() {
            return None;
        }
        Some(VisibilityProjectionView::new(
            self.runtime(),
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis),
        ))
    }
}

#[path = "historical_denial.rs"]
mod historical_denial;
use historical_denial::historical_projection_denial;
