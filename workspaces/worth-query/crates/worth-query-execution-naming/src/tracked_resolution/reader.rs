//! The tracked persistent-name reader at one pinned observation.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::facade::application_schema::{
    fragment, ApplicationEntityMarkerIdentity,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::lineage::{
    LineageRecordTarget, PersistentNameObservationView, PersistentNameOfRecord,
    PersistentNameSubjectKind,
};
use worth_relational::facade::mvcc::RelationalBranchObservation;
use worth_relational::facade::runtime::RelationalRuntime;

use super::{ExecutionNameReadDenial, NameReadDependencies};
use crate::admission::{kind_verified_resolved_name, AtObservation, RequestedSubject};

/// Unprivileged, dependency-tracked persistent-name reads exactly as one
/// admitted branch observation sees them.
///
/// The reader owns no state of its own: each read admits the observation
/// once, reads Relational's derived name index at the observation's commit,
/// and records what the answer rested on into the caller's
/// [`NameReadDependencies`].
pub struct ExecutionNameReader<'read> {
    pub(super) runtime: &'read RelationalRuntime,
    pub(super) execution: &'read ExecutionRequest<'read, 'read>,
    pub(super) observation: &'read RelationalBranchObservation,
}

impl<'read> ExecutionNameReader<'read> {
    pub fn at_observation(
        runtime: &'read RelationalRuntime,
        execution: &'read ExecutionRequest<'read, 'read>,
        observation: &'read RelationalBranchObservation,
    ) -> Self {
        Self {
            runtime,
            execution,
            observation,
        }
    }

    /// The branch every read and recorded dependency belongs to.
    pub fn branch(&self) -> &BranchId {
        self.observation.identity().branch_id()
    }

    /// The admitted `WorthQueryName<Entity>` the live entity carries.
    ///
    /// Refuses typed when the entity carries no name, when its name no longer
    /// designates it, or when the name's installed scheme governs another
    /// subject than `Entity`. Records the entity's lineage.
    ///
    /// Complexity: O(1) expected index probes.
    pub fn entity_name_of<Schema, Entity>(
        &self,
        entity: EntityId,
        dependencies: &mut NameReadDependencies,
    ) -> Result<WorthQueryName<Entity>, ExecutionNameReadDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        let requested = RequestedSubject {
            kind: PersistentNameSubjectKind::Entity,
            name: Entity::IDENTIFIER,
        };
        self.name_of(LineageRecordTarget::Entity(entity), requested, dependencies)
    }

    /// The admitted `WorthQueryName<Relation>` the live relation carries.
    ///
    /// Complexity: O(1) expected index probes.
    pub fn relation_name_of<Relation>(
        &self,
        relation: RelationId,
        dependencies: &mut NameReadDependencies,
    ) -> Result<WorthQueryName<Relation>, ExecutionNameReadDenial>
    where
        Relation: fragment::Relation,
    {
        let requested = RequestedSubject {
            kind: PersistentNameSubjectKind::Relation,
            name: Relation::NAME,
        };
        self.name_of(
            LineageRecordTarget::Relation(relation),
            requested,
            dependencies,
        )
    }

    fn name_of<K>(
        &self,
        record: LineageRecordTarget,
        requested: RequestedSubject<'_>,
        dependencies: &mut NameReadDependencies,
    ) -> Result<WorthQueryName<K>, ExecutionNameReadDenial> {
        let branch = self.branch().clone();
        let (name, lineage) = self.with_view(|view| match view.name_of_record(record) {
            PersistentNameOfRecord::Live { name, lineage } => kind_verified_resolved_name::<K>(
                &AtObservation {
                    runtime: self.runtime,
                    view: &view,
                },
                name,
                requested,
            )
            .map(|admitted| (admitted, lineage)),
            PersistentNameOfRecord::Ended { .. } => {
                Err(ExecutionNameReadDenial::RecordNoLongerNamed { record })
            }
            PersistentNameOfRecord::Unnamed => {
                Err(ExecutionNameReadDenial::RecordNotNamed { record })
            }
        })??;
        dependencies.record_live_record(&branch, lineage, record);
        Ok(name)
    }

    /// Runs `read` over the derived name index at the observation's commit.
    pub(super) fn with_view<T>(
        &self,
        read: impl FnOnce(PersistentNameObservationView<'_>) -> T,
    ) -> Result<T, ExecutionNameReadDenial> {
        self.runtime
            .with_persistent_names_at_observation(self.execution, self.observation, read)
            .map_err(ExecutionNameReadDenial::Relational)
    }
}
