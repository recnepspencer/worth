//! Migration preserves prior performed effects; lifecycle succession does not.
use super::{register_relation, relations, WorthQueryPrimaryGraphInstallationDenial};
use worth_relational::facade::{
    identity::KindId,
    schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId},
};

pub(super) fn register(
    registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
    instance: KindId,
    transition: KindId,
    migrated_from: KindId,
    prior_effect: KindId,
) -> Result<RelationalSchemaRegistry, WorthQueryPrimaryGraphInstallationDenial> {
    let registry = register_relation(
        registry,
        schema,
        version,
        "worth-query-workflow-instance-migrated-from",
        migrated_from,
        instance,
        instance,
        relations::single_successor_integrity(),
    )?;
    register_relation(
        registry,
        schema,
        version,
        "worth-query-workflow-instance-prior-effect",
        prior_effect,
        instance,
        transition,
        relations::prior_effect_integrity(),
    )
}
