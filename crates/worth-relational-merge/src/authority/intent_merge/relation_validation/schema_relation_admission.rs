use crate::contracts::schema::data::RelationalSchemaRegistry;
use crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict;

use super::super::schema_conflicts::schema_error_to_commit_conflict;

pub(super) fn require_registered_relation_kind(
    schema: &RelationalSchemaRegistry,
    kind_id: crate::contracts::identity::data::primitives::KindId,
) -> Result<&crate::contracts::schema::data::RelationKindRegistration, CommitConflict> {
    let schema_registry = schema;
    schema_registry
        .resolve_relation(kind_id)
        .map_err(schema_error_to_commit_conflict)?;
    schema_registry
        .relation_registration(kind_id)
        .map_err(schema_error_to_commit_conflict)
}
