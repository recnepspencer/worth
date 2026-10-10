//! Memory runtime construction policy.
use super::*;

pub(super) fn memory_runtime_builder(
    registry: RelationalSchemaRegistry,
    invariant_catalog: InvariantCatalog,
    initial_seed_rows: Option<usize>,
) -> worth_relational::facade::runtime::RelationalRuntimeBuilder {
    let builder = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .invariant_catalog(invariant_catalog.canonicalized());
    let Some(initial_seed_rows) = initial_seed_rows.filter(|rows| *rows > 0) else {
        return builder;
    };
    let defaults = RelationalRuntimeConfig::resolved(
        RelationalRuntimeProfile::CertificationCore,
        RelationalConfigOverride::default(),
    );
    let mut publication = defaults.publication.policy;
    publication.max_patch_records_per_commit = publication
        .max_patch_records_per_commit
        .max(initial_seed_rows);
    builder
        .entity_capacity(
            defaults
                .storage
                .initial_entity_capacity
                .max(initial_seed_rows),
        )
        .publication(publication)
}
