use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::runtime::VisibilityProjectionView;
use crate::schema::data::{AspectContractPlanCatalog, LoweredAspectContractPlan, SchemaVersionId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

/// Schema-qualified storage projection used to derive or certify an index.
///
/// Construction is private to this module so a caller cannot pair records
/// with an ambient or unrelated schema authority.
pub(in crate::indexes) struct IndexProjectionSource<'view, 'runtime> {
    projection: &'view VisibilityProjectionView<'runtime>,
}

impl<'view, 'runtime> IndexProjectionSource<'view, 'runtime> {
    pub(in crate::indexes) fn selected(
        projection: &'view VisibilityProjectionView<'runtime>,
    ) -> Self {
        Self { projection }
    }
    pub(in crate::indexes) fn exact(
        projection: &'view VisibilityProjectionView<'runtime>,
    ) -> Option<Self> {
        projection.is_exact_basis().then_some(Self { projection })
    }

    pub(in crate::indexes) fn historical(
        projection: &'view VisibilityProjectionView<'runtime>,
    ) -> Option<Self> {
        (!projection.is_exact_basis()).then_some(Self { projection })
    }

    fn schema_authority(&self) -> Option<&RelationalBranchRootSchemaAuthority> {
        self.projection.selected_schema_authority()
    }

    pub(in crate::indexes) fn schema_version(&self) -> Option<SchemaVersionId> {
        self.schema_authority()
            .map(RelationalBranchRootSchemaAuthority::schema_version)
    }

    pub(in crate::indexes) fn aspect_plans(&self) -> Option<&AspectContractPlanCatalog> {
        self.schema_authority()
            .map(RelationalBranchRootSchemaAuthority::aspect_plans)
    }

    pub(in crate::indexes) fn entity_aspect_plan(
        &self,
        kind_id: KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.schema_authority()?.entity_aspect_plan(kind_id)
    }

    pub(in crate::indexes) fn relation_aspect_plan(
        &self,
        kind_id: KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.schema_authority()?.relation_aspect_plan(kind_id)
    }

    pub(in crate::indexes) fn for_each_entity(
        &self,
        kind_id: KindId,
        mut visit: impl FnMut(&EntityReadRecord),
    ) {
        for record in self.projection.authoritative_entity_records(kind_id) {
            visit(&record);
        }
    }

    pub(in crate::indexes) fn for_each_relation(
        &self,
        kind_id: KindId,
        mut visit: impl FnMut(&RelationReadRecord),
    ) {
        for record in self.projection.authoritative_relation_records(kind_id) {
            visit(&record);
        }
    }

    pub(in crate::indexes) fn with_entity<T>(
        &self,
        entity_id: EntityId,
        inspect: impl FnOnce(&EntityReadRecord) -> T,
    ) -> Option<T> {
        self.projection
            .authoritative_entity_record(entity_id)
            .as_ref()
            .map(inspect)
    }

    pub(in crate::indexes) fn with_relation<T>(
        &self,
        relation_id: RelationId,
        inspect: impl FnOnce(&RelationReadRecord) -> T,
    ) -> Option<T> {
        self.projection
            .authoritative_relation_record(relation_id)
            .as_ref()
            .map(inspect)
    }
}
