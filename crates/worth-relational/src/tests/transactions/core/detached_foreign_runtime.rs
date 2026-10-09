use crate::facade::transactions::{
    CreateIntent, EntityReference, MutationIntent, RelationSpec, WorkerIntentBatch,
};
use crate::tests::support::*;

#[test]
fn foreign_runtime_validation_denials_precede_raw_entity_and_relation_effects() {
    let runtime_a = runtime_with_test_schema();

    let mut invariant_transaction = test_owner_begin_transaction_for_main(&runtime_a);
    stage_raw_entity_relation_graph(&mut invariant_transaction, "invariant");
    let invariant_target = require_interned_runtime();
    let invariant_target_before = TargetRuntimeState::capture(&invariant_target);
    let invariant_error = invariant_transaction
        .commit_boundary_plan(&invariant_target)
        .expect_err("invariant planning cannot cross its runtime boundary");
    assert!(matches!(
        invariant_error.class,
        crate::facade::transactions::ConflictClass::ForeignRuntime { .. }
    ));
    invariant_target_before.assert_unchanged(&invariant_target);

    let mut transaction = test_owner_begin_transaction_for_main(&runtime_a);
    stage_raw_entity_relation_graph(&mut transaction, "validation");
    let validation_target = require_interned_runtime();
    let validation_target_before = TargetRuntimeState::capture(&validation_target);
    let error = match transaction.validate(
        &validation_target,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ) {
        Err(error) => error,
        Ok(_) => panic!("a transaction cannot cross its runtime boundary"),
    };
    assert!(matches!(
        error,
        crate::facade::transactions::TransactionCommitError::Conflict { error, .. }
            if matches!(
                error.class,
                crate::facade::transactions::ConflictClass::ForeignRuntime { .. }
            )
    ));
    validation_target_before.assert_unchanged(&validation_target);
}

#[test]
fn foreign_runtime_validated_proposal_denial_preserves_exact_target_state() {
    let runtime_a = runtime_with_test_schema();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime_a);
    stage_raw_entity_relation_graph(&mut transaction, "validated");
    let candidate = transaction
        .validate(
            &runtime_a,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("source runtime validates its own transaction");
    let runtime_b = require_interned_runtime();
    let target_before = TargetRuntimeState::capture(&runtime_b);

    let error = runtime_b
        .prepare_validated_proposal(candidate)
        .expect_err("validated authority cannot cross its runtime boundary");
    assert!(matches!(
        error,
        crate::facade::transactions::TransactionCommitError::Conflict { error, .. }
            if matches!(
                error.class,
                crate::facade::transactions::ConflictClass::ForeignRuntime { .. }
            )
    ));
    target_before.assert_unchanged(&runtime_b);
}

#[test]
fn foreign_runtime_discard_denies_without_public_residue_and_disposes_candidate() {
    let runtime_a = runtime_with_test_schema();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime_a);
    transaction
        .push_batch(
            batch_create("prepared-owner-affinity"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime_a
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the source owner prepares its own candidate");
    let source_before = TargetRuntimeState::capture(&runtime_a);

    let runtime_b = require_interned_runtime();
    let target_before = TargetRuntimeState::capture(&runtime_b);
    let error = runtime_b
        .discard_prepared_candidate(candidate)
        .expect_err("foreign discard denies owner mismatch while consuming the candidate");

    assert!(matches!(
        error,
        crate::facade::transactions::TransactionCommitError::Conflict { error, .. }
            if matches!(
                error.class,
                crate::facade::transactions::ConflictClass::ForeignRuntime { .. }
            )
    ));
    source_before.assert_unchanged(&runtime_a);
    target_before.assert_unchanged(&runtime_b);
}

fn require_interned_runtime() -> crate::facade::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .client_key_symbol_policy(ClientKeySymbolPolicy::RequireInterned)
        .build()
}

fn stage_raw_entity_relation_graph(
    transaction: &mut crate::facade::mvcc::BranchBoundRelationalTransaction,
    prefix: &str,
) {
    let source_key = format!("{prefix}-source");
    let target_key = format!("{prefix}-target");
    let source = created_entity(&source_key);
    let target = created_entity(&target_key);
    transaction
        .push_batch(
            batch_create(&source_key),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            batch_create(&target_key),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            WorkerIntentBatch::new(format!("{prefix}-relation-batch")).push(
                MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                    partition_id: crate::facade::identity::PartitionId::main(),
                    kind_id: crate::facade::identity::KindId(2),
                    client_key: crate::facade::symbols::ClientKey::raw(format!(
                        "{prefix}-relation"
                    )),
                    source: EntityReference::Created(source),
                    target: EntityReference::Created(target),
                    fields: Default::default(),
                })),
            ),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
}

fn created_entity(client_key: &str) -> crate::facade::transactions::CreatedEntityRef {
    crate::facade::transactions::CreatedEntityRef {
        partition_id: crate::facade::identity::PartitionId::main(),
        kind_id: crate::facade::identity::KindId(1),
        client_key: crate::facade::symbols::ClientKey::raw(client_key),
    }
}

struct TargetRuntimeState {
    symbols: crate::symbols::data::StringInterner,
    configured_symbols: crate::symbols::data::SymbolTableSnapshot,
    branch_cells: Vec<crate::branch::RelationalBranchCellCheckpoint>,
    catalog: Vec<crate::history::data::CanonicalCommitEnvelope>,
    reference_cost: crate::runtime::RelationalPhase4ReferenceCostCounters,
    complexity: crate::performance::data::RuntimeComplexityCounters,
}

impl TargetRuntimeState {
    fn capture(runtime: &crate::facade::runtime::RelationalRuntime) -> Self {
        Self {
            symbols: runtime.services.symbols.interner_snapshot(),
            configured_symbols: runtime.config().identity.symbol_table.clone(),
            branch_cells: runtime.history().branch_cells_snapshot(),
            catalog: runtime.history().commit_envelopes_snapshot(),
            reference_cost: runtime.phase4_reference_cost_counters(),
            complexity: runtime.performance_access().counters(),
        }
    }

    fn assert_unchanged(&self, runtime: &crate::facade::runtime::RelationalRuntime) {
        assert_eq!(runtime.services.symbols.interner_snapshot(), self.symbols);
        assert_eq!(
            runtime.config().identity.symbol_table,
            self.configured_symbols
        );
        assert_eq!(
            runtime.phase4_reference_cost_counters(),
            self.reference_cost
        );
        assert_eq!(runtime.performance_access().counters(), self.complexity);
        assert_eq!(runtime.history().branch_cells_snapshot(), self.branch_cells);
        assert_eq!(runtime.history().commit_envelopes_snapshot(), self.catalog);
    }
}
