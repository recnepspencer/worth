//! Migration preserves prior performed effects; lifecycle succession does not.
use {crate::domain_computation::graph_layout::register_relation, crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial};
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
        &crate::domain_computation::graph_layout::workflow::relations::single_successor_integrity().into(),
        None,
    )?;
    register_relation(
        registry,
        schema,
        version,
        "worth-query-workflow-instance-prior-effect",
        prior_effect,
        instance,
        transition,
        &crate::domain_computation::graph_layout::workflow::relations::prior_effect_integrity().into(),
        None,
    )
}
