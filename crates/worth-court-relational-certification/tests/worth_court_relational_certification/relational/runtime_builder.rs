//! Public runtime builder profile customization.
use worth_relational::facade::{
    config::{DurableLogPolicy, DurableLogRetentionMode},
    diagnostics::RelationalDiagnosticsProfile,
    runtime::RelationalRuntimeBuilder,
    schema::RelationalSchemaRegistry,
};

#[test]
fn builder_applies_explicit_settings_over_profile_defaults() {
    let runtime = RelationalRuntimeBuilder::new()
        .runtime_name("configured-runtime")
        .diagnostics(RelationalDiagnosticsProfile::geometry_operational_hot_path())
        .schema_registry(RelationalSchemaRegistry::new())
        .entity_capacity(42)
        .relation_capacity(24)
        .durable_log_policy(DurableLogPolicy {
            retention_mode: DurableLogRetentionMode::CompactAfterCheckpoint,
            max_in_memory_envelopes: 7,
            compact_after_checkpoint: true,
        })
        .build()
        .expect("valid native runtime configuration");

    assert_eq!(
        runtime.config().execution.runtime_name,
        "configured-runtime"
    );
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
