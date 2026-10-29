#[allow(unused_imports)] use crate::relational_certification::ordinary_context::invariant_branch_selection::*;
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_custom_invariant, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {std::sync::Arc};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::PartitionId};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::runtime::CustomInvariantDescriptor, worth_relational::facade::runtime::CustomInvariantExecutionContext, worth_relational::facade::runtime::CustomInvariantExecutionError, worth_relational::facade::runtime::CustomInvariantOperationalMetadata, worth_relational::facade::runtime::CustomInvariantPreparationError, worth_relational::facade::runtime::CustomInvariantRegistration, worth_relational::facade::runtime::CustomInvariantRule, worth_relational::facade::runtime::CustomInvariantScopePlanner, worth_relational::facade::runtime::CustomInvariantSemanticIdentity, worth_relational::facade::runtime::CustomInvariantSemanticVersion, worth_relational::facade::runtime::CustomInvariantVerdict, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantFailureEffect, worth_relational::facade::runtime::InvariantGroup, worth_relational::facade::runtime::InvariantGroupSet, worth_relational::facade::runtime::InvariantReportedRule, worth_relational::facade::runtime::InvariantRule};
pub(crate) use {worth_relational::facade::schema::CardinalityContractDeclaration, worth_relational::facade::schema::ContractId, worth_relational::facade::schema::MinimumCardinalityEnforcement, worth_relational::facade::schema::RelationIntegrityDeclarations, worth_relational::facade::schema::RelationalSchemaRegistry};
pub(crate) use {worth_relational::facade::symbols::ClientKey};
pub(crate) use {worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RelationSpec, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch};

#[test]
fn supply_chain_child_commit_uses_child_root_after_main_advances() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let original = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let registry = registry_with_vessel_source_cardinality(original.schema_registry());
    let program = original.with_schema_registry_for_test(registry);
    let world = compile_supply_chain_baseline(execution, program).expect("Court world compiles");

    let source = world.handles.entities
        [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel, 1)]
        .id;
    let main_target = world.handles.entities
        [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 1)]
        .id;
    let child_target = world.handles.entities
        [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 2)]
        .id;

    let (_, source_observation) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    world
        .runtime
        .fork_branch(execution, BranchId("storm".to_owned()), source_observation)
        .expect("storm fork shares the Supply Chain root");

    let main_commit = commit_relation(
        execution,
        &world.runtime,
        BranchId("main".to_owned()),
        source,
        main_target,
        "main-vessel-one-assignment",
    )
    .expect("main assignment commits against the advanced main root");
    assert_native_cardinality_execution(&main_commit);

    let main_head_before_rejection =
        head_for_supply_chain_branch(&world.runtime, &BranchId("main".to_owned())).version_id;
    let rejected_main_assignment = commit_relation(
        execution,
        &world.runtime,
        BranchId("main".to_owned()),
        source,
        child_target,
        "main-vessel-one-conflicting-assignment",
    );
    assert!(
        rejected_main_assignment.is_err(),
        "native source-cardinality invariant must reject the second main assignment"
    );
    let main_head_after_rejection =
        head_for_supply_chain_branch(&world.runtime, &BranchId("main".to_owned())).version_id;
    assert_eq!(main_head_after_rejection, main_head_before_rejection);

    let child_commit = commit_relation(
        execution,
        &world.runtime,
        BranchId("storm".to_owned()),
        source,
        child_target,
        "storm-vessel-one-assignment",
    );
    assert!(
        child_commit.is_ok(),
        "the child root has no assignment; a global-main invariant route would reject it: {child_commit:?}"
    );
    assert_native_cardinality_execution(&child_commit.expect("child assignment remains legal"));
}

#[test]
fn supply_chain_child_custom_invariant_uses_child_selected_version() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let original = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let registry = registry_with_vessel_source_cardinality(original.schema_registry());
    let program = original.with_schema_registry_for_test(registry);
    let world = compile_supply_chain_baseline_with_custom_invariant(
        execution,
        program,
        worth_relational::facade::runtime::CustomInvariantRegistration::new(BranchVersionProbeRule)
            .expect("branch-version custom invariant registers"),
    )
    .expect("Court world compiles with the custom invariant");

    let source = world.handles.entities
        [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel, 1)]
        .id;
    let main_target = world.handles.entities
        [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 1)]
        .id;
    let child_target = world.handles.entities
        [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 2)]
        .id;

    let (_, source_observation) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    world
        .runtime
        .fork_branch(execution, BranchId("storm".to_owned()), source_observation)
        .expect("storm fork shares the Supply Chain root");

    let main_commit = commit_relation(
        execution,
        &world.runtime,
        BranchId("main".to_owned()),
        source,
        main_target,
        "main-vessel-one-custom-assignment",
    )
    .expect("main assignment commits against the advanced main root");
    let child_basis_version =
        head_for_supply_chain_branch(&world.runtime, &BranchId("storm".to_owned())).version_id;
    assert_ne!(child_basis_version, main_commit.version_id);

    let child_commit = commit_relation(
        execution,
        &world.runtime,
        BranchId("storm".to_owned()),
        source,
        child_target,
        "storm-vessel-one-custom-assignment",
    )
    .expect("storm commit uses its own root and custom invariant");
    let custom_execution = child_commit
        .invariant_executions()
        .iter()
        .find_map(|execution| {
            execution.results().iter().find_map(|result| {
                matches!(
                    &result.rule,
                    worth_relational::facade::runtime::InvariantReportedRule::Custom(identity)
                        if identity.rule_id.as_str() == crate::relational_certification::ordinary_context::invariant_branch_selection::BranchVersionProbeRule::RULE_ID
                )
                .then_some((execution, result))
            })
        })
        .expect("child commit records the registered custom invariant");
    assert_eq!(
        custom_execution.0.metadata().execution_point(),
        worth_relational::facade::runtime::InvariantExecutionPoint::CommitBoundary
    );
    assert_eq!(
        custom_execution.0.metadata().current_version_id(),
        child_basis_version,
        "custom metadata must preserve the branch-selected basis version"
    );
    assert_eq!(
        custom_execution
            .1
            .custom_provenance()
            .expect("custom execution carries provenance")
            .current_version_id,
        child_basis_version,
        "custom provenance must preserve the branch-selected basis version"
    );
}
