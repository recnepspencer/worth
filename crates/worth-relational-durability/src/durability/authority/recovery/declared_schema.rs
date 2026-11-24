//! D5: the schema a recovered runtime serves is the one its recovered main
//! branch head records — the initial schema, or the target of the last durable
//! schema transition. A runtime that declares any other registry, including
//! one with identical contracts under a different version label, is refused
//! before service: only a durable schema transition changes what it serves.

use contracts::durability::data::recovery_errors::{
    DurabilityError, RecoveryAuthorityContinuityMismatch, RecoveryFailureClass,
};
use contracts::schema::data::authority_snapshot::carried_projection::SchemaAuthoritySnapshot;
use worth_relational_contracts::internal_port as contracts;

/// Refuse `rebuilt` unless its declared registry is exactly the schema
/// authority its recovered main head records.
///
/// Cost: one authority snapshot of the declared registry, O(kinds).
pub fn require_declared_schema_of_recovered_main(
    recorded: &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
    declared_registry: &contracts::schema::data::RelationalSchemaRegistry,
) -> Result<(), DurabilityError> {
    let declared = declared_registry.authority_snapshot();
    if recorded.matches(&declared) {
        return Ok(());
    }
    let recorded_registry = recorded.registry();
    let recorded_snapshot = recorded_registry.authority_snapshot();
    Err(DurabilityError::new(
        RecoveryFailureClass::DeclaredSchemaMismatch,
        format!(
            "the declared schema {} is not the schema {} the recovered main head records",
            describe(&declared),
            describe(&recorded_snapshot),
        ),
    )
    .with_authority_continuity_mismatch(
        RecoveryAuthorityContinuityMismatch::SchemaRegistryShape {
            expected_primary_schema_version: recorded_registry.primary_schema_version_id(),
            found_primary_schema_version: declared_registry.primary_schema_version_id(),
            expected_entity_kind_count: recorded_snapshot.entity_kinds.len(),
            found_entity_kind_count: declared.entity_kinds.len(),
            expected_relation_kind_count: recorded_snapshot.relation_kinds.len(),
            found_relation_kind_count: declared.relation_kinds.len(),
        },
    ))
}

fn describe(snapshot: &SchemaAuthoritySnapshot) -> String {
    match (
        &snapshot.primary_schema_id,
        snapshot.primary_schema_version_id,
    ) {
        (Some(id), Some(version)) => format!("`{}` v{}", id.0, version.0),
        _ => "with no single primary schema".to_owned(),
    }
}
