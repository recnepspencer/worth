#[allow(unused_imports)] use crate::relational_certification::ordinary_context::invariant_global_uniqueness::*;
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_uniqueness_assertion::assert_unique_conflict};
pub(crate) use {std::collections::BTreeMap};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_invariant_catalog, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::insert_vessel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::next_vessel_key, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::vessel_call_signs as oracle_vessel_call_signs, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::UniqueEntityFieldOracleError};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::ContractValidatedAspectValueView, worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString};
pub(crate) use {worth_relational::facade::runtime::InvariantCatalog, worth_relational::facade::runtime::InvariantRegistration, worth_relational::facade::runtime::InvariantReportedRule, worth_relational::facade::runtime::InvariantRule, worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch};
pub(crate) use {worth_relational::facade::history::BranchId, worth_relational::facade::identity::PartitionId, worth_relational::facade::symbols::ClientKey};

#[test]
fn global_uniqueness_rejects_duplicate_on_one_branch_without_residue() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let program = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let catalog = InvariantCatalog {
        registrations: vec![
            worth_relational::facade::runtime::InvariantRegistration::commit_boundary_blocking(
                worth_relational::facade::runtime::InvariantRule::unique_entity_aspect_field(
                    worth_foundational::facade::AspectKey::new("call_sign")
                        .expect("call-sign aspect"),
                    worth_foundational::facade::FieldKey::new("call_sign")
                        .expect("call-sign field"),
                ),
            ),
        ],
    };
    let world = compile_supply_chain_baseline_with_invariant_catalog(execution, program, catalog)
        .expect("Court world with native uniqueness compiles");
    let main = BranchId("main".to_owned());
    commit_vessel(execution, &world.runtime, main.clone(), "first-vessel")
        .expect("first branch vessel commits");
    let before_values = vessel_call_signs(execution, &world.runtime, &main);
    let before_branch = world
        .runtime
        .branch_reference_state(&main)
        .expect("main branch reference is observable");
    let before_version = world
        .runtime
        .history()
        .historical_latest_commit()
        .expect("baseline has a latest commit")
        .version_id;
    let before_catalog = world.runtime.history().immutable_commit_count();
    let before_snapshot = current_snapshot_version(&world.runtime, &main);

    let duplicate = commit_vessel(execution, &world.runtime, main.clone(), "duplicate-vessel");
    assert_unique_conflict(duplicate.unwrap_err(), DIVERGENT_CALL_SIGN);

    assert_eq!(
        vessel_call_signs(execution, &world.runtime, &main),
        before_values
    );
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&main)
            .expect("baseline branch reference remains observable"),
        before_branch,
        "rejected duplicate must not leave a branch-reference residue"
    );
    assert_eq!(
        world
            .runtime
            .history()
            .historical_latest_commit()
            .expect("baseline commit remains latest after rejection")
            .version_id,
        before_version
    );
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        before_catalog
    );
    assert_eq!(
        current_snapshot_version(&world.runtime, &main),
        before_snapshot
    );
}

#[test]
fn global_uniqueness_rejects_two_colliding_creates_in_one_proposal_without_residue() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let program = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let catalog = InvariantCatalog {
        registrations: vec![
            worth_relational::facade::runtime::InvariantRegistration::commit_boundary_blocking(
                worth_relational::facade::runtime::InvariantRule::unique_entity_aspect_field(
                    worth_foundational::facade::AspectKey::new("call_sign")
                        .expect("call-sign aspect"),
                    worth_foundational::facade::FieldKey::new("call_sign")
                        .expect("call-sign field"),
                ),
            ),
        ],
    };
    let world = compile_supply_chain_baseline_with_invariant_catalog(execution, program, catalog)
        .expect("Court world with native uniqueness compiles");
    let main = BranchId("main".to_owned());
    let before_values = vessel_call_signs(execution, &world.runtime, &main);
    let before_branch = world
        .runtime
        .branch_reference_state(&main)
        .expect("main branch reference is observable");
    let before_catalog = world.runtime.history().immutable_commit_count();
    let before_snapshot = current_snapshot_version(&world.runtime, &main);

    let duplicate = commit_two_vessels(execution, &world.runtime, main.clone());
    assert_unique_conflict(duplicate.unwrap_err(), DIVERGENT_CALL_SIGN);

    assert_eq!(
        vessel_call_signs(execution, &world.runtime, &main),
        before_values
    );
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&main)
            .expect("main branch reference remains observable"),
        before_branch,
        "rejected proposal must not advance the branch reference"
    );
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        before_catalog
    );
    assert_eq!(
        current_snapshot_version(&world.runtime, &main),
        before_snapshot
    );
}

#[allow(unused_imports)]
pub(crate) use {};
pub(crate) use {crate::relational_certification::ordinary_context::invariant_global_uniqueness::current_version::current_snapshot_version};
