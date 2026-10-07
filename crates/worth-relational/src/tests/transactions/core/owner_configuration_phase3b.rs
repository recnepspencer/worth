use crate::runtime::RelationalPreparationOwnerBinding;
use crate::tests::support::*;

/// Installing the initial schema is owner authority, and a live settlement
/// service does not withdraw it.
///
/// The service holds a weak binding to this exact runtime, which is precisely
/// the shape that used to make an owner's own configuration change impossible.
#[test]
fn phase3b_initial_schema_installs_while_a_settlement_port_is_live() {
    let mut runtime = RelationalRuntimeApi::builder().build();
    let port = runtime.settlement_port();

    let receipt = runtime
        .prepare_initial_schema_installation()
        .expect("an uncommitted runtime still holds initial schema authority")
        .install(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .expect("the owner installs its initial schema while a service is bound to it");

    assert_eq!(receipt.retained_entity_kind_count(), 1);
    assert_eq!(receipt.retained_relation_kind_count(), 1);
    assert!(runtime
        .config()
        .schema
        .registry
        .entity_kinds
        .contains_key(&KindId(1)));
    assert!(
        runtime.entity_aspect_plan_trace(KindId(1)).is_some(),
        "the contract runtime is lowered from the registry that was installed",
    );
    assert_eq!(port.runtime_instance_id(), runtime.runtime_instance_id());
}

/// The installed schema and the contract runtime lowered from it are one
/// change. A port bound before the installation never observes the new registry
/// against the old contract runtime, and nothing has to be reconciled after the
/// fact for it to see either.
#[test]
fn phase3b_installed_schema_is_the_only_configuration_authority() {
    let mut runtime = RelationalRuntimeApi::builder().build();
    let preparation = RelationalPreparationOwnerBinding::from_runtime(&runtime);
    let empty = preparation.runtime_snapshot();
    assert!(empty.config.schema.registry.entity_kinds.is_empty());
    assert!(empty
        .schema_contract_runtime
        .aspect_contract_plans
        .entity_plans
        .is_empty());

    runtime
        .prepare_initial_schema_installation()
        .expect("an uncommitted runtime still holds initial schema authority")
        .install(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .expect("the owner installs its initial schema");

    let installed = preparation.runtime_snapshot();
    assert!(installed
        .config
        .schema
        .registry
        .entity_kinds
        .contains_key(&KindId(1)));
    assert!(
        installed
            .schema_contract_runtime
            .aspect_contract_plans
            .entity_plans
            .contains_key(&KindId(1)),
        "the registry and the plans lowered from it arrive together",
    );
    assert!(
        empty
            .schema_contract_runtime
            .aspect_contract_plans
            .entity_plans
            .is_empty(),
        "the snapshot taken before the installation is not rewritten by it",
    );
}
