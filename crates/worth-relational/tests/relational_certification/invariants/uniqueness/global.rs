use crate::invariant_uniqueness_assertion::assert_unique_conflict;
use std::collections::BTreeMap;

use super::world::supply_chain::{
    compile_supply_chain_baseline_with_invariant_catalog, entity_kind_id, insert_vessel,
    next_vessel_key, snapshot_for_supply_chain_identity,
    vessel_call_signs as oracle_vessel_call_signs, CompiledSupplyChainProgram, EntityKind,
    OracleBranch, OracleState, SupplyChainScale, SupplyChainWorldDefinition,
    UniqueEntityFieldOracleError,
};
use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidatedAspectValueView, FieldKey, InternedString,
};
use worth_relational::facade::runtime::{
    InvariantCatalog, InvariantRegistration, InvariantReportedRule, InvariantRule,
    RelationalRuntime,
};
use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent,
    TransactionCommitError, WorkerIntentBatch,
};
use worth_relational::facade::{history::BranchId, identity::PartitionId, symbols::ClientKey};

const DIVERGENT_CALL_SIGN: &str = "BRANCH-DIVERGENCE";

#[test]
fn global_uniqueness_uses_selected_branch_proposal_after_sibling_diverges() {
    let program = CompiledSupplyChainProgram::compile(
        SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(
                AspectKey::new("call_sign").expect("call-sign aspect"),
                FieldKey::new("call_sign").expect("call-sign field"),
            ),
        )],
    };
    let world = compile_supply_chain_baseline_with_invariant_catalog(program, catalog)
        .expect("Court world with native uniqueness compiles");
    let main = BranchId("main".to_owned());
    let storm = BranchId("storm".to_owned());
    let (_, source) = world
        .runtime
        .observe_fork_source(&main)
        .expect("main remains forkable");
    world
        .runtime
        .fork_branch(storm.clone(), source)
        .expect("storm fork shares the baseline root");

    let oracle_genesis = OracleBranch::genesis(OracleState::from_definition(
        &SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    ));
    let oracle_storm = oracle_genesis
        .fork(
            super::world::supply_chain::BranchLabel::Storm,
            super::world::supply_chain::BranchLabel::Operating,
        )
        .expect("oracle storm branch forks from operating");
    let oracle_main = insert_vessel(
        &oracle_genesis,
        next_vessel_key(&oracle_genesis),
        DIVERGENT_CALL_SIGN,
    )
    .expect("oracle accepts the main branch vessel");
    let oracle_storm = insert_vessel(
        &oracle_storm,
        next_vessel_key(&oracle_storm),
        DIVERGENT_CALL_SIGN,
    )
    .expect("oracle accepts the independent storm vessel");

    let main_commit = commit_vessel(&world.runtime, main.clone(), "main-only-vessel");
    assert!(main_commit.is_ok(), "main-only call sign is admissible");
    assert_unique_execution(main_commit.as_ref().expect("main commit"));
    let main_values = vessel_call_signs(&world.runtime, &main);
    let storm_before = vessel_call_signs(&world.runtime, &storm);
    assert_eq!(main_values, oracle_vessel_call_signs(&oracle_main));
    assert_eq!(storm_before, oracle_vessel_call_signs(&oracle_genesis));

    // A global-current-state mutant would see main's value here and reject this
    // legal child commit. The semantic oracle is independent of production
    // snapshots, indexes, and record identifiers.
    let storm_commit = commit_vessel(&world.runtime, storm.clone(), "storm-only-vessel");
    assert!(
        storm_commit.is_ok(),
        "child uniqueness must use the child root plus its proposal: {storm_commit:?}"
    );
    assert_unique_execution(storm_commit.as_ref().expect("storm commit"));
    let storm_values = vessel_call_signs(&world.runtime, &storm);
    assert_eq!(storm_values, oracle_vessel_call_signs(&oracle_storm));
    assert_eq!(main_values, oracle_vessel_call_signs(&oracle_main));
}

#[test]
fn global_uniqueness_rejects_duplicate_on_one_branch_without_residue() {
    let program = CompiledSupplyChainProgram::compile(
        SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(
                AspectKey::new("call_sign").expect("call-sign aspect"),
                FieldKey::new("call_sign").expect("call-sign field"),
            ),
        )],
    };
    let world = compile_supply_chain_baseline_with_invariant_catalog(program, catalog)
        .expect("Court world with native uniqueness compiles");
    let main = BranchId("main".to_owned());
    commit_vessel(&world.runtime, main.clone(), "first-vessel")
        .expect("first branch vessel commits");
    let before_values = vessel_call_signs(&world.runtime, &main);
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

    let duplicate = commit_vessel(&world.runtime, main.clone(), "duplicate-vessel");
    assert_unique_conflict(duplicate.unwrap_err(), DIVERGENT_CALL_SIGN);

    assert_eq!(vessel_call_signs(&world.runtime, &main), before_values);
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
    let program = CompiledSupplyChainProgram::compile(
        SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(
                AspectKey::new("call_sign").expect("call-sign aspect"),
                FieldKey::new("call_sign").expect("call-sign field"),
            ),
        )],
    };
    let world = compile_supply_chain_baseline_with_invariant_catalog(program, catalog)
        .expect("Court world with native uniqueness compiles");
    let main = BranchId("main".to_owned());
    let before_values = vessel_call_signs(&world.runtime, &main);
    let before_branch = world
        .runtime
        .branch_reference_state(&main)
        .expect("main branch reference is observable");
    let before_catalog = world.runtime.history().immutable_commit_count();
    let before_snapshot = current_snapshot_version(&world.runtime, &main);

    let duplicate = commit_two_vessels(&world.runtime, main.clone());
    assert_unique_conflict(duplicate.unwrap_err(), DIVERGENT_CALL_SIGN);

    assert_eq!(vessel_call_signs(&world.runtime, &main), before_values);
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

#[test]
fn semantic_uniqueness_oracle_rejects_duplicate_without_mutating_branch() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let genesis = OracleBranch::genesis(OracleState::from_definition(&definition));
    let key = next_vessel_key(&genesis);
    let first =
        insert_vessel(&genesis, key, DIVERGENT_CALL_SIGN).expect("oracle accepts first value");
    let duplicate = insert_vessel(&first, next_vessel_key(&first), DIVERGENT_CALL_SIGN);
    assert_eq!(
        duplicate,
        Err(UniqueEntityFieldOracleError::DuplicateValue(
            DIVERGENT_CALL_SIGN.to_owned()
        ))
    );
    assert_eq!(
        oracle_vessel_call_signs(&first),
        oracle_vessel_call_signs(&genesis)
            .into_iter()
            .chain([DIVERGENT_CALL_SIGN.to_owned()])
            .collect::<Vec<_>>()
    );
}

fn commit_vessel(
    runtime: &RelationalRuntime,
    branch: BranchId,
    client_key: &str,
) -> Result<worth_relational::facade::transactions::CommitResult, TransactionCommitError> {
    let identity = runtime
        .branch_identity(&branch)
        .expect("branch identity is owner-issued");
    let options = runtime
        .admit_branch_basis(&identity)
        .expect("transaction authority is owner-issued");
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    push_vessel(&mut transaction, client_key);
    transaction.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}

fn commit_two_vessels(
    runtime: &RelationalRuntime,
    branch: BranchId,
) -> Result<worth_relational::facade::transactions::CommitResult, TransactionCommitError> {
    let identity = runtime
        .branch_identity(&branch)
        .expect("branch identity is owner-issued");
    let options = runtime
        .admit_branch_basis(&identity)
        .expect("transaction authority is owner-issued");
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    push_vessel(&mut transaction, "same-transaction-vessel-a");
    push_vessel(&mut transaction, "same-transaction-vessel-b");
    transaction.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}

fn push_vessel(
    transaction: &mut worth_relational::facade::mvcc::BranchBoundRelationalTransaction,
    client_key: &str,
) {
    transaction
        .push_batch(
            WorkerIntentBatch::new(client_key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: entity_kind_id(EntityKind::Vessel),
                    client_key: ClientKey::raw(client_key),
                    fields: vessel_fields(DIVERGENT_CALL_SIGN),
                },
            ))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
}

fn vessel_fields(call_sign: &str) -> AspectFieldPatch {
    let mut fields = BTreeMap::new();
    insert_string(&mut fields, "call_sign", call_sign);
    insert_string(&mut fields, "class", "Feeder");
    fields.insert(
        planned_single_field_locator(
            AspectKey::new("capacity").expect("capacity aspect"),
            FieldKey::new("capacity").expect("capacity field"),
        ),
        AspectValue::UInt64(9_999),
    );
    insert_string(&mut fields, "posture", "Open");
    AspectFieldPatch::new(fields)
}

fn insert_string(
    fields: &mut BTreeMap<worth_foundational::facade::AspectFieldLocator, AspectValue>,
    name: &str,
    value: &str,
) {
    fields.insert(
        planned_single_field_locator(
            AspectKey::new(name).expect("aspect key"),
            FieldKey::new(name).expect("field key"),
        ),
        AspectValue::String(InternedString::Raw(value.to_owned())),
    );
}

fn vessel_call_signs(runtime: &RelationalRuntime, branch: &BranchId) -> Vec<String> {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    let snapshot = snapshot_for_supply_chain_identity(runtime, &identity);
    let view = runtime
        .read_truth()
        .read_snapshot(&snapshot)
        .expect("branch snapshot is readable");
    let aspect = AspectKey::new("call_sign").expect("call-sign aspect");
    view.entities()
        .iter()
        .filter(|record| record.kind.kind_id == entity_kind_id(EntityKind::Vessel))
        .filter_map(|record| {
            let state = record.authoritative_aspect_state.as_ref()?;
            let value = state.get(&aspect)?.view();
            match value {
                ContractValidatedAspectValueView::Scalar(AspectValue::String(
                    InternedString::Raw(value),
                )) => Some(value.clone()),
                ContractValidatedAspectValueView::Scalar(_)
                | ContractValidatedAspectValueView::Struct(_) => None,
            }
        })
        .collect()
}
fn assert_unique_execution(commit: &worth_relational::facade::transactions::CommitResult) {
    let mut observed = false;
    for execution in commit.invariant_executions() {
        if execution.results().iter().any(|result| {
            matches!(
                &result.rule,
                InvariantReportedRule::Native(InvariantRule::UniqueEntityAspectField { .. })
            )
        }) {
            let identity = execution
                .metadata()
                .proposal_identity()
                .expect("committed uniqueness execution carries proposal identity");
            assert_eq!(identity.proposed_version_id(), commit.version_id);
            observed = true;
        }
    }
    assert!(observed, "commit must record native uniqueness execution");
}

fn current_snapshot_version(runtime: &RelationalRuntime, branch: &BranchId) -> u64 {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    snapshot_for_supply_chain_identity(runtime, &identity)
        .version_id()
        .0
}
