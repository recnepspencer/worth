pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_uniqueness_assertion::assert_unique_conflict};
pub(crate) use {std::collections::BTreeMap};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_invariant_catalog, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::insert_vessel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::next_vessel_key, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::vessel_call_signs as oracle_vessel_call_signs, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::UniqueEntityFieldOracleError};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::ContractValidatedAspectValueView, worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString};
pub(crate) use {worth_relational::facade::runtime::InvariantCatalog, worth_relational::facade::runtime::InvariantRegistration, worth_relational::facade::runtime::InvariantReportedRule, worth_relational::facade::runtime::InvariantRule, worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch};
pub(crate) use {worth_relational::facade::history::BranchId, worth_relational::facade::identity::PartitionId, worth_relational::facade::symbols::ClientKey};

pub(crate) const DIVERGENT_CALL_SIGN: &str = "BRANCH-DIVERGENCE";

pub(crate) fn commit_vessel(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    transaction.commit(execution, runtime)
}

pub(crate) fn commit_two_vessels(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    transaction.commit(execution, runtime)
}

pub(crate) fn push_vessel(
    transaction: &mut worth_relational::facade::mvcc::BranchBoundRelationalTransaction,
    client_key: &str,
) {
    transaction
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new(client_key).push(worth_relational::facade::transactions::MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::Entity(
                EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(client_key),
                    fields: vessel_fields(DIVERGENT_CALL_SIGN),
                },
            ))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
}

pub(crate) fn vessel_fields(call_sign: &str) -> AspectFieldPatch {
    let mut fields = std::collections::BTreeMap::new();
    insert_string(&mut fields, "call_sign", call_sign);
    insert_string(&mut fields, "class", "Feeder");
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new("capacity").expect("capacity aspect"),
            worth_foundational::facade::FieldKey::new("capacity").expect("capacity field"),
        ),
        worth_foundational::facade::AspectValue::UInt64(9_999),
    );
    insert_string(&mut fields, "posture", "Open");
    worth_relational::facade::transactions::AspectFieldPatch::new(fields)
}

pub(crate) fn insert_string(
    fields: &mut BTreeMap<worth_foundational::facade::AspectFieldLocator, AspectValue>,
    name: &str,
    value: &str,
) {
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new(name).expect("aspect key"),
            worth_foundational::facade::FieldKey::new(name).expect("field key"),
        ),
        worth_foundational::facade::AspectValue::String(worth_foundational::facade::InternedString::Raw(value.to_owned())),
    );
}

pub(crate) fn vessel_call_signs(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> Vec<String> {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    let snapshot = snapshot_for_supply_chain_identity(runtime, &identity);
    let view = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("fixture read request remains live")
        .expect("branch snapshot is readable");
    let aspect = worth_foundational::facade::AspectKey::new("call_sign").expect("call-sign aspect");
    view.entities()
        .iter()
        .filter(|record| record.kind.kind_id == entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel))
        .filter_map(|record| {
            let state = record.authoritative_aspect_state.as_ref()?;
            let value = state.get(&aspect)?.view();
            match value {
                worth_foundational::facade::ContractValidatedAspectValueView::Scalar(worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw(value),
                )) => Some(value.clone()),
                worth_foundational::facade::ContractValidatedAspectValueView::Scalar(_)
                | worth_foundational::facade::ContractValidatedAspectValueView::Struct(_) => None,
            }
        })
        .collect()
}
pub(crate) fn assert_unique_execution(
    commit: &worth_relational::facade::transactions::CommitResult,
) {
    let mut observed = false;
    for execution in commit.invariant_executions() {
        if execution.results().iter().any(|result| {
            matches!(
                &result.rule,
                worth_relational::facade::runtime::InvariantReportedRule::Native(worth_relational::facade::runtime::InvariantRule::UniqueEntityAspectField { .. })
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


#[allow(unused_imports)]
pub(crate) use {};
pub(crate) use {crate::relational_certification::ordinary_context::invariant_global_uniqueness::current_version::current_snapshot_version};