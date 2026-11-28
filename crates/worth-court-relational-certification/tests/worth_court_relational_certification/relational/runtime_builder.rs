//! Public runtime builder grouped setup overrides.
use worth_relational::facade::{
    config::{ConfigValueSource, DurableLogPolicy, DurableLogRetentionMode},
    diagnostics::RelationalDiagnosticsProfile,
    runtime::RelationalRuntimeBuilder,
    schema::RelationalSchemaRegistry,
};

#[test]
fn builder_grouped_setup_sections_apply_overrides() {
    let runtime = RelationalRuntimeBuilder::new()
        .runtime_setup(|runtime| {
            runtime
                .runtime_name("grouped-runtime")
                .diagnostics(RelationalDiagnosticsProfile::geometry_operational_hot_path());
        })
        .schema_setup(|schema| {
            schema.schema_registry(RelationalSchemaRegistry::new());
        })
        .storage_setup(|storage| {
            storage.entity_capacity(42).relation_capacity(24);
        })
        .durability_setup(|durability| {
            durability.durable_log_policy(DurableLogPolicy {
                retention_mode: DurableLogRetentionMode::CompactAfterCheckpoint,
                max_in_memory_envelopes: 7,
                compact_after_checkpoint: true,
            });
        })
        .build()
        .expect("valid native runtime configuration");

    assert_eq!(runtime.config().execution.runtime_name, "grouped-runtime");
    assert_eq!(runtime.config().storage.initial_entity_capacity, 42);
    assert_eq!(runtime.config().storage.initial_relation_capacity, 24);
    assert_eq!(
        runtime
            .config()
            .durability
            .policy
            .log
            .max_in_memory_envelopes,
        7
    );
    assert_eq!(
        runtime.config().diagnostics.profile,
        RelationalDiagnosticsProfile::geometry_operational_hot_path()
    );
}
