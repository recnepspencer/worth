use std::collections::BTreeMap;

use crate::config::data::*;

mod commit_strategy_registration_digest;

use commit_strategy_registration_digest::commit_strategy_registration_set_digest_hex;

pub(super) fn apply_config_overrides(
    config: &mut RelationalRuntimeConfig,
    overrides: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    apply_execution_overrides(config, overrides, provenance);
    apply_diagnostics_overrides(config, overrides, provenance);
    apply_history_overrides(config, overrides, provenance);
    apply_schema_overrides(config, overrides, provenance);
    apply_commit_strategy_overrides(config, overrides, provenance);
    apply_identity_overrides(config, overrides, provenance);
    apply_storage_overrides(config, overrides, provenance);
    apply_visibility_overrides(config, overrides, provenance);
    apply_publication_overrides(config, overrides, provenance);
    apply_durability_overrides(config, overrides, provenance);
}

fn apply_execution_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.execution;
    insert_override_provenance(
        provenance,
        "execution.runtime_name",
        section.runtime_name.is_some(),
    );
    insert_override_provenance(
        provenance,
        "execution.compiled_lane_policy",
        section.compiled_lane_policy.is_some(),
    );
    insert_override_provenance(
        provenance,
        "execution.relation_integrity_scope_budget",
        section.relation_integrity_scope_budget.is_some(),
    );

    if let Some(runtime_name) = &section.runtime_name {
        config.execution.runtime_name = runtime_name.clone();
    }
    if let Some(compiled_lane_policy) = section.compiled_lane_policy {
        config.execution.compiled_lane_policy = compiled_lane_policy;
    }
    if let Some(relation_integrity_scope_budget) = &section.relation_integrity_scope_budget {
        config.execution.relation_integrity_scope_budget = relation_integrity_scope_budget.clone();
    }
}

fn apply_diagnostics_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.diagnostics;
    insert_override_provenance(provenance, "diagnostics.profile", section.profile.is_some());
    if let Some(profile) = &section.profile {
        config.diagnostics.profile = profile.clone();
    }
}

fn apply_history_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.history;
    insert_override_provenance(
        provenance,
        "history.version_graph_policy",
        section.version_graph_policy.is_some(),
    );
    insert_override_provenance(provenance, "history.retention", section.retention.is_some());
    insert_override_provenance(
        provenance,
        "history.main_branch",
        section.main_branch.is_some(),
    );
    if let Some(version_graph_policy) = section.version_graph_policy {
        config.history.version_graph_policy = version_graph_policy;
    }
    if let Some(retention) = section.retention {
        config.history.retention = retention;
    }
    if let Some(main_branch) = &section.main_branch {
        config.history.main_branch = main_branch.clone();
    }
}

fn apply_schema_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.schema;
    insert_override_provenance(provenance, "schema.registry", section.registry.is_some());
    insert_override_provenance(
        provenance,
        "schema.invariant_catalog",
        section.invariant_catalog.is_some(),
    );
    if let Some(registry) = &section.registry {
        config.schema.registry = registry.clone();
    }
    if let Some(invariant_catalog) = &section.invariant_catalog {
        config.schema.invariant_catalog = invariant_catalog.clone();
    }
}

fn apply_identity_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.identity;
    insert_override_provenance(
        provenance,
        "identity.client_key_symbol_policy",
        section.client_key_symbol_policy.is_some(),
    );
    insert_override_provenance(
        provenance,
        "identity.symbol_table",
        section.symbol_table.is_some(),
    );
    if let Some(client_key_symbol_policy) = section.client_key_symbol_policy {
        config.identity.client_key_symbol_policy = client_key_symbol_policy;
    }
    if let Some(symbol_table) = &section.symbol_table {
        config.identity.symbol_table = symbol_table.clone();
    }
}

fn apply_commit_strategy_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.commit_strategies;
    if let Some(registrations) = &section.registrations {
        config.commit_strategies.registrations = registrations.clone();
        provenance.insert(
            "commit_strategies.registrations".to_string(),
            ConfigProvenanceEntry {
                source: ConfigValueSource::BuilderOverride,
                detail: commit_strategy_override_detail(registrations),
            },
        );
    } else {
        provenance.insert(
            "commit_strategies.registrations".to_string(),
            ConfigProvenanceEntry {
                source: ConfigValueSource::ProfileDefault,
                detail: commit_strategy_override_detail(&config.commit_strategies.registrations),
            },
        );
    }
}

fn apply_storage_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.storage;
    insert_override_provenance(
        provenance,
        "storage.initial_entity_capacity",
        section.initial_entity_capacity.is_some(),
    );
    insert_override_provenance(
        provenance,
        "storage.initial_relation_capacity",
        section.initial_relation_capacity.is_some(),
    );
    insert_override_provenance(provenance, "storage.mvcc", section.mvcc.is_some());
    insert_override_provenance(provenance, "storage.retention", section.retention.is_some());
    insert_override_provenance(provenance, "storage.layout", section.layout.is_some());
    insert_override_provenance(
        provenance,
        "storage.adjacency_policy",
        section.adjacency_policy.is_some(),
    );
    insert_override_provenance(
        provenance,
        "storage.cross_context_policy",
        section.cross_context_policy.is_some(),
    );
    insert_override_provenance(
        provenance,
        "storage.cascade_delete_policy",
        section.cascade_delete_policy.is_some(),
    );

    if let Some(capacity) = section.initial_entity_capacity {
        config.storage.initial_entity_capacity = capacity;
    }
    if let Some(capacity) = section.initial_relation_capacity {
        config.storage.initial_relation_capacity = capacity;
    }
    if let Some(mvcc) = &section.mvcc {
        config.storage.mvcc = mvcc.clone();
        config.storage.retention.backend = mvcc.retention_backend;
        config.storage.retention.reclaim_batch_size = mvcc.reclaim_batch_size;
    }
    if let Some(retention) = &section.retention {
        config.storage.retention = retention.clone();
    }
    if let Some(layout) = &section.layout {
        config.storage.layout = layout.clone();
    }
    if let Some(adjacency_policy) = &section.adjacency_policy {
        config.storage.adjacency_policy = adjacency_policy.clone();
    }
    if let Some(cross_context_policy) = section.cross_context_policy {
        config.storage.cross_context_policy = cross_context_policy;
    }
    if let Some(cascade_delete_policy) = section.cascade_delete_policy {
        config.storage.cascade_delete_policy = cascade_delete_policy;
    }
}

fn apply_visibility_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.visibility;
    insert_override_provenance(
        provenance,
        "visibility.cache_policy",
        section.cache_policy.is_some(),
    );
    if let Some(cache_policy) = &section.cache_policy {
        config.visibility.cache_policy = cache_policy.clone();
    }
}

fn apply_publication_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.publication;
    insert_override_provenance(provenance, "publication.policy", section.policy.is_some());
    if let Some(policy) = &section.policy {
        config.publication.policy = policy.clone();
    }
}

fn apply_durability_overrides(
    config: &mut RelationalRuntimeConfig,
    override_tree: &RelationalConfigOverride,
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
) {
    let section = &override_tree.durability;
    insert_override_provenance(provenance, "durability.policy", section.policy.is_some());
    insert_override_provenance(provenance, "durability.mode", section.mode.is_some());
    insert_override_provenance(provenance, "durability.log", section.log.is_some());
    insert_override_provenance(
        provenance,
        "durability.store_layout",
        section.store_layout.is_some(),
    );
    if let Some(policy) = &section.policy {
        config.durability.policy = policy.clone();
    }
    if let Some(mode) = section.mode {
        config.durability.policy.mode = mode;
    }
    if let Some(log) = &section.log {
        config.durability.policy.log = log.clone();
        config
            .durability
            .policy
            .checkpoints
            .compact_after_checkpoint = log.compact_after_checkpoint;
    }
    if let Some(store_layout) = &section.store_layout {
        config.durability.policy.store_layout = Some(store_layout.clone());
    }
}

fn insert_override_provenance(
    provenance: &mut BTreeMap<String, ConfigProvenanceEntry>,
    path: &str,
    overridden: bool,
) {
    provenance.insert(path.to_string(), provenance_entry(overridden));
}

fn provenance_entry(overridden: bool) -> ConfigProvenanceEntry {
    if overridden {
        ConfigProvenanceEntry {
            source: ConfigValueSource::BuilderOverride,
            detail: "explicit builder override".to_string(),
        }
    } else {
        ConfigProvenanceEntry {
            source: ConfigValueSource::ProfileDefault,
            detail: "resolved from runtime profile".to_string(),
        }
    }
}

fn commit_strategy_override_detail(
    registrations: &[crate::commit_strategies::data::CommitStrategyRegistration],
) -> String {
    let digest = commit_strategy_registration_set_digest_hex(registrations);
    format!(
        "commit strategy registrations resolved; count={}; descriptor_set_digest={digest}",
        registrations.len()
    )
}
