use super::*;
use crate::identity::data::{EntityId, RelationId, VersionId};

impl super::ProjectionRecordSource for VisibilityProjectionView<'_> {
    fn selected_root(&self) -> Option<&std::sync::Arc<crate::branch::RelationalBranchRoot>> {
        self.selected_root()
    }
    fn is_exact_basis(&self) -> bool {
        self.is_exact_basis()
    }
    fn version_id(&self) -> VersionId {
        self.version_id()
    }
    fn authoritative_entity_record(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        id: EntityId,
    ) -> Option<crate::storage::data::EntityReadRecord> {
        self.authoritative_entity_record(request, id)
    }
    fn authoritative_relation_record(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        id: RelationId,
    ) -> Option<crate::storage::data::RelationReadRecord> {
        self.authoritative_relation_record(request, id)
    }
}
