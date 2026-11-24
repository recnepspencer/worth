use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;

use super::super::schema_conflicts::schema_error_to_commit_conflict;

pub(super) fn require_registered_relation_kind(
    schema: &RelationalSchemaRegistry,
    kind_id: crate::identity::data::KindId,
) -> Result<&crate::schema::data::RelationKindRegistration, CommitConflict> {
    let schema_registry = schema;
    schema_registry
        .resolve_relation(kind_id)
        .map_err(schema_error_to_commit_conflict)?;
    schema_registry
        .relation_registration(kind_id)
        .map_err(schema_error_to_commit_conflict)
}
