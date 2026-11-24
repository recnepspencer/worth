//! Supplies the empty invariant environment for unit tests of the evaluator and custom rules.
use super::{InvariantRuntimeInputs, InvariantRuntimeView};
use crate::config::data::RelationalRuntimeConfig;
use crate::identity::data::VersionId;
use crate::runtime::state::subsystems::schema_contract_runtime::SchemaContractRuntimeSubsystem;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use std::sync::Arc;

pub(crate) fn empty_invariant_view(
    instrumentation: &RuntimeInstrumentation,
) -> InvariantRuntimeView<'_> {
    invariant_view(
        instrumentation,
        RelationalRuntimeConfig::default(),
        VersionId(0),
    )
}

pub(crate) fn invariant_view(
    instrumentation: &RuntimeInstrumentation,
    config: RelationalRuntimeConfig,
    version: VersionId,
) -> InvariantRuntimeView<'_> {
    invariant_view_with_custom_rules(instrumentation, config, version, Vec::new())
}

pub(crate) fn invariant_view_with_custom_rules(
    instrumentation: &RuntimeInstrumentation,
    config: RelationalRuntimeConfig,
    version: VersionId,
    rules: Vec<crate::validation::custom_rule::CustomInvariantRegistration>,
) -> InvariantRuntimeView<'_> {
    let config = Arc::new(config);
    let control = crate::schema::SchemaCompilationControl::Configuration;
    let rules = crate::validation::custom_registry::FrozenCustomInvariantRegistry::compile(
        &config.schema.registry,
        rules,
        Vec::new(),
        control,
    )
    .unwrap();
    let plans = crate::branch::root::RetainedRelationalAspectPlans::compile(
        &config.schema.registry,
        control,
    )
    .unwrap();
    let schema_contract_runtime = Arc::new(SchemaContractRuntimeSubsystem::for_schema(
        &config.schema.registry,
        &config.schema.invariant_catalog.registrations,
        plans,
        rules,
        0,
        false,
    ));
    InvariantRuntimeView::from_inputs(InvariantRuntimeInputs {
        selected_registry: crate::schema::SchemaRegistryCustody::Runtime(Arc::clone(&config)),
        prior_registry: None,
        commit_work_budget: None,
        config,
        schema_contract_runtime,
        instrumentation,
        current_version_id: version,
        entity_count: 0,
        relation_count: 0,
        version_depth: 0,
        snapshot_pressure: false,
    })
}
