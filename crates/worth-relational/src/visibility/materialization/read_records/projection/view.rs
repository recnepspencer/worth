use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;

use super::super::reader::VisibilityReadContext;
use super::contracts::{assert_declared_projection_aspects, ProjectionAspectScope};
use super::projection_records::{
    EntityProjectionRecord, EntityRecordProjection, RelationProjectionRecord,
    RelationRecordProjection,
};

#[derive(Debug, Clone)]
pub struct VisibilityProjectionView<'runtime> {
    pub(super) runtime: &'runtime RelationalRuntime,
    pub(super) basis: crate::visibility::snapshot_states::SnapshotStateBasis,
}

impl<'runtime> VisibilityProjectionView<'runtime> {
    pub(crate) fn new(
        runtime: &'runtime RelationalRuntime,
        basis: crate::visibility::snapshot_states::SnapshotStateBasis,
    ) -> Self {
        Self { runtime, basis }
    }

    pub const fn version_id(&self) -> VersionId {
        self.basis.version_id()
    }

    pub(crate) fn is_exact_basis(&self) -> bool {
        matches!(
            self.basis,
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(_)
        )
    }

    pub(crate) fn selected_schema_authority(
        &self,
    ) -> Option<&crate::branch::RelationalBranchRootSchemaAuthority> {
        self.basis.root().map(|root| root.schema_authority())
    }

    pub fn entities<T: EntityRecordProjection>(&self) -> Vec<T> {
        let projection_scope = self.assert_entity_projection_contract::<T>();
        self.authoritative_entity_records(T::KIND)
            .into_iter()
            .filter_map(|record| {
                T::from_record(EntityProjectionRecord::new(&record, &projection_scope))
            })
            .collect()
    }

    pub fn entities_in<T: EntityRecordProjection>(&self, partition_id: PartitionId) -> Vec<T> {
        let projection_scope = self.assert_entity_projection_contract::<T>();
        self.authoritative_entity_records_in(partition_id, T::KIND)
            .into_iter()
            .filter_map(|record| {
                T::from_record(EntityProjectionRecord::new(&record, &projection_scope))
            })
            .collect()
    }

    pub fn entity<T: EntityRecordProjection>(&self, entity_id: EntityId) -> Option<T> {
        let projection_scope = self.assert_entity_projection_contract::<T>();
        self.project_entity_record(entity_id, &projection_scope, T::from_record)
    }

    pub fn entity_records_with_projection_scope<T>(
        &self,
        kind_id: KindId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Vec<T> {
        self.assert_entity_projection_scope(kind_id, &projection_scope);
        self.authoritative_entity_records(kind_id)
            .into_iter()
            .filter_map(|record| project(EntityProjectionRecord::new(&record, &projection_scope)))
            .collect()
    }

    pub fn entity_record_with_projection_scope<T>(
        &self,
        entity_id: EntityId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Option<T> {
        self.project_entity_record(entity_id, &projection_scope, |record| {
            self.assert_entity_projection_scope(record.kind_id(), &projection_scope);
            project(record)
        })
    }

    pub(crate) fn entity_record_of_expected_kind_with_projection_scope<T>(
        &self,
        entity_id: EntityId,
        expected_kind_id: KindId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Result<Option<T>, KindId> {
        self.project_entity_record(entity_id, &projection_scope, |record| {
            Some(if record.kind_id() != expected_kind_id {
                Err(record.kind_id())
            } else {
                self.assert_entity_projection_scope(expected_kind_id, &projection_scope);
                Ok(project(record))
            })
        })
        .unwrap_or(Ok(None))
    }

    pub fn relations<T: RelationRecordProjection>(&self) -> Vec<T> {
        let projection_scope = self.assert_relation_projection_contract::<T>();
        self.authoritative_relation_records(T::KIND)
            .into_iter()
            .filter_map(|record| {
                T::from_record(RelationProjectionRecord::new(&record, &projection_scope))
            })
            .collect()
    }

    pub fn relations_in<T: RelationRecordProjection>(&self, partition_id: PartitionId) -> Vec<T> {
        let projection_scope = self.assert_relation_projection_contract::<T>();
        self.authoritative_relation_records_in(partition_id, T::KIND)
            .into_iter()
            .filter_map(|record| {
                T::from_record(RelationProjectionRecord::new(&record, &projection_scope))
            })
            .collect()
    }

    pub fn relation<T: RelationRecordProjection>(&self, relation_id: RelationId) -> Option<T> {
        let projection_scope = self.assert_relation_projection_contract::<T>();
        self.authoritative_relation_record(relation_id)
            .and_then(|record| {
                T::from_record(RelationProjectionRecord::new(&record, &projection_scope))
            })
    }

    pub fn relation_records_with_projection_scope<T>(
        &self,
        kind_id: KindId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(RelationProjectionRecord<'_>) -> Option<T>,
    ) -> Vec<T> {
        self.assert_relation_projection_scope(kind_id, &projection_scope);
        self.authoritative_relation_records(kind_id)
            .into_iter()
            .filter_map(|record| project(RelationProjectionRecord::new(&record, &projection_scope)))
            .collect()
    }

    pub fn relation_record_with_projection_scope<T>(
        &self,
        relation_id: RelationId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(RelationProjectionRecord<'_>) -> Option<T>,
    ) -> Option<T> {
        let record = self.authoritative_relation_record(relation_id)?;
        self.assert_relation_projection_scope(record.kind.kind_id, &projection_scope);
        project(RelationProjectionRecord::new(&record, &projection_scope))
    }

    pub(super) fn reader(&self) -> VisibilityReadContext<'runtime> {
        VisibilityReadContext::new(self.runtime)
    }

    fn assert_entity_projection_contract<T: EntityRecordProjection>(
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

    fn assert_relation_projection_contract<T: RelationRecordProjection>(
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

    fn assert_entity_projection_scope(
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

    fn assert_relation_projection_scope(
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

    fn entity_aspect_plan(
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

    fn relation_aspect_plan(
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
    /// Project through an owner-issued exact branch observation without
    /// allocating a tracked snapshot handle.
    pub fn project_observation(
        &self,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<VisibilityProjectionView<'runtime>, crate::branch::RelationalBranchBasisDenial>
    {
        if observation.identity().runtime_instance_id() != self.runtime().runtime_instance_id() {
            return Err(crate::branch::RelationalBranchBasisDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime().runtime_instance_id(),
                actual_runtime_instance_id: observation.identity().runtime_instance_id(),
            });
        }
        if observation
            .selected_root()
            .has_materialization_unavailable()
        {
            return Err(crate::branch::RelationalBranchBasisDenial::MaterializationUnavailable);
        }
        let basis = crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_observation(
            observation,
        );
        Ok(VisibilityProjectionView::new(
            self.runtime(),
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis),
        ))
    }

    pub(crate) fn project_branch_head(
        &self,
        branch_id: &crate::history::data::BranchId,
        version_id: VersionId,
    ) -> Result<
        Option<VisibilityProjectionView<'runtime>>,
        crate::branch::RelationalBranchBasisDenial,
    > {
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
    ) -> VisibilityProjectionView<'runtime> {
        self.try_project_historical_version(version_id)
            .expect("internal historical projection requires retained MVCC coverage")
    }

    pub(crate) fn try_project_historical_version(
        &self,
        version_id: VersionId,
    ) -> Result<VisibilityProjectionView<'runtime>, crate::branch::RelationalBranchBasisDenial>
    {
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

    pub fn project_snapshot(
        &self,
        handle: &SnapshotHandle,
    ) -> Option<VisibilityProjectionView<'runtime>> {
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

fn historical_projection_denial(
    denial: crate::visibility::snapshot_states::HistoricalVisibilityDenial,
) -> crate::branch::RelationalBranchBasisDenial {
    use crate::visibility::snapshot_states::HistoricalVisibilityDenial as Denial;
    match denial {
        Denial::RetentionCapacityExhausted => {
            crate::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted
        }
        Denial::RetentionIdentityExhausted => {
            crate::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted
        }
        Denial::RetentionUnavailable => crate::branch::RelationalBranchBasisDenial::OwnerFailure,
        Denial::UnknownVersion
        | Denial::AuthoringBranchUnavailable
        | Denial::MvccIntervalUnavailable
        | Denial::CertificationReconstructionRequired => {
            crate::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget
        }
    }
}
