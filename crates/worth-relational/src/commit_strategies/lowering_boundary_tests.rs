use super::lower_execution;
use super::tests::{canonical_request, execution_draft, execution_draft_with_batches};

use crate::facade::history::BranchId;
use crate::facade::transactions::{
    CreateIntent, CreatedEntityRef, EntityMutationIntent, EntityReference, EntitySpec,
    MutationIntent, RecordRef, RelationSpec, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::ClientKey;

#[test]
fn stale_strategy_basis_denies_before_raw_key_normalization() {
    let runtime = crate::runtime::builder::RelationalRuntimeBuilder::new()
        .schema_registry(crate::tests::support::test_schema_registry())
        .client_key_symbol_policy(crate::symbols::data::ClientKeySymbolPolicy::RequireInterned)
        .build();
    let validation_input =
        crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime);
    let transaction = runtime
        .begin_branch_transaction_with_owner_inputs(validation_input)
        .expect("owner context opens a branch-bound transaction");
    crate::tests::support::create_entity_outcome(&runtime, "strategy-basis-advance");
    let symbols_before = runtime.services.symbols.clone();
    let configured_symbols_before = runtime.config().identity.symbol_table.clone();
    let branch_cells_before = runtime.history().branch_cells_snapshot();
    let catalog_before = runtime.history().commit_envelopes_snapshot();
    let reference_cost_before = runtime.phase4_reference_cost_counters();
    let complexity_before = runtime.performance_access().counters();
    let request = canonical_request();
    let execution = execution_draft(&request);

    let error = lower_execution(&runtime, &request, &execution, transaction)
        .expect_err("stale basis denies before strategy key normalization");

    assert!(matches!(
        error,
        crate::commit_strategies::data::StrategyLoweringError::MutationConflict(conflict)
            if matches!(
                conflict.class,
                crate::transactions::data::ConflictClass::StaleValidationBasis { .. }
            )
    ));
    assert_eq!(runtime.services.symbols, symbols_before);
    assert_eq!(
        runtime.config().identity.symbol_table,
        configured_symbols_before
    );
    assert_eq!(
        runtime.phase4_reference_cost_counters(),
        reference_cost_before
    );
    assert_eq!(runtime.performance_access().counters(), complexity_before);
    assert_eq!(
        runtime.history().branch_cells_snapshot(),
        branch_cells_before
    );
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        catalog_before
    );
}

#[test]
fn foreign_strategy_basis_preserves_taxonomy_and_exact_target_state() {
    let runtime_a = crate::tests::support::runtime_with_test_schema();
    let validation_input =
        crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime_a);
    let runtime_b = crate::runtime::builder::RelationalRuntimeBuilder::new()
        .schema_registry(crate::tests::support::test_schema_registry())
        .client_key_symbol_policy(crate::symbols::data::ClientKeySymbolPolicy::RequireInterned)
        .build();
    let symbols_before = runtime_b.services.symbols.clone();
    let configured_symbols_before = runtime_b.config().identity.symbol_table.clone();
    let branch_cells_before = runtime_b.history().branch_cells_snapshot();
    let catalog_before = runtime_b.history().commit_envelopes_snapshot();
    let reference_cost_before = runtime_b.phase4_reference_cost_counters();
    let complexity_before = runtime_b.performance_access().counters();
    let request = canonical_request();
    let execution = execution_draft(&request);

    let error = crate::commit_strategies::facade::CommitStrategiesAuthorityFacade::new()
        .lower_execution_with_input(&runtime_b, &request, &execution, validation_input)
        .expect_err("foreign strategy basis is rejected at admission");

    assert!(matches!(
        error,
        crate::commit_strategies::data::StrategyLoweringError::MutationConflict(conflict)
            if matches!(
                conflict.class,
                crate::transactions::data::ConflictClass::ForeignRuntime { .. }
            )
    ));
    assert_eq!(runtime_b.services.symbols, symbols_before);
    assert_eq!(
        runtime_b.config().identity.symbol_table,
        configured_symbols_before
    );
    assert_eq!(
        runtime_b.phase4_reference_cost_counters(),
        reference_cost_before
    );
    assert_eq!(runtime_b.performance_access().counters(), complexity_before);
    assert_eq!(
        runtime_b.history().branch_cells_snapshot(),
        branch_cells_before
    );
    assert_eq!(
        runtime_b.history().commit_envelopes_snapshot(),
        catalog_before
    );
}

#[test]
fn sibling_strategy_target_denies_before_normalization_with_zero_residue() {
    let runtime = require_interned_runtime();
    crate::tests::support::create_entity_outcome(&runtime, "shared-strategy-root");
    fork_from_main(&runtime, "strategy-storm");
    fork_from_main(&runtime, "strategy-maintenance");
    let mut storm = begin_on(&runtime, "strategy-storm");
    storm
        .push_batch(
            create_batch("storm-only-strategy-entity"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let storm_outcome = storm
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("storm create commits");
    let storm_only = storm_outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(entity) => Some(*entity),
            RecordRef::Relation(_) => None,
        })
        .expect("storm commit creates one entity");
    let transaction = begin_on(&runtime, "strategy-maintenance");
    let request = canonical_request();
    let execution = execution_draft_with_batches(
        &request,
        vec![
            create_batch("must-not-be-interned-strategy"),
            WorkerIntentBatch::new("sibling-strategy-target").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: storm_only,
                    fields: crate::tests::support::name_field_patch("must-not-apply"),
                }),
            )),
        ],
    );
    let before = RuntimeState::capture(&runtime);

    let error = lower_execution(&runtime, &request, &execution, transaction)
        .expect_err("strategy lowering rejects sibling-only targets");

    assert!(matches!(
        error,
        crate::commit_strategies::data::StrategyLoweringError::MutationConflict(conflict)
            if matches!(
                conflict.class,
                crate::transactions::data::ConflictClass::StaleTarget { .. }
            )
    ));
    before.assert_unchanged(&runtime);
}

#[test]
fn unowned_strategy_created_endpoint_denies_with_zero_residue() {
    let runtime = require_interned_runtime();
    let transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    let request = canonical_request();
    let missing = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw("missing-strategy-created-endpoint"),
    };
    let execution = execution_draft_with_batches(
        &request,
        vec![
            WorkerIntentBatch::new("unowned-created-endpoint").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw("must-not-be-interned-strategy-edge"),
                    source: EntityReference::Created(missing.clone()),
                    target: EntityReference::Created(missing),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                }),
            )),
        ],
    );
    let before = RuntimeState::capture(&runtime);

    let error = lower_execution(&runtime, &request, &execution, transaction)
        .expect_err("strategy created endpoints must belong to the transaction");

    assert!(matches!(
        error,
        crate::commit_strategies::data::StrategyLoweringError::MutationConflict(conflict)
            if matches!(
                conflict.class,
                crate::transactions::data::ConflictClass::InvalidRelationEndpoint { .. }
            )
    ));
    before.assert_unchanged(&runtime);
}

fn require_interned_runtime() -> crate::runtime::RelationalRuntime {
    crate::runtime::builder::RelationalRuntimeBuilder::new()
        .schema_registry(crate::tests::support::test_schema_registry())
        .client_key_symbol_policy(crate::symbols::data::ClientKeySymbolPolicy::RequireInterned)
        .build()
}

fn create_batch(client_key: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new(client_key).push(MutationIntent::Create(CreateIntent::Entity(
        EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(client_key),
            fields: crate::tests::support::name_field_patch(client_key),
        },
    )))
}

fn fork_from_main(runtime: &crate::runtime::RelationalRuntime, branch: &str) {
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has a committed fork source");
    runtime
        .fork_branch(BranchId(branch.to_owned()), basis)
        .expect("branch fork succeeds");
}

fn begin_on(
    runtime: &crate::runtime::RelationalRuntime,
    branch: &str,
) -> crate::mvcc::BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("branch identity exists");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("branch basis is admitted");
    runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .expect("basis belongs to this runtime")
}

struct RuntimeState {
    symbols: crate::symbols::data::StringInterner,
    configured_symbols: crate::symbols::data::SymbolTableSnapshot,
    branch_cells: Vec<crate::branch::RelationalBranchCellCheckpoint>,
    catalog: Vec<crate::history::data::CanonicalCommitEnvelope>,
    reference_cost: crate::runtime::RelationalPhase4ReferenceCostCounters,
    complexity: crate::performance::data::RuntimeComplexityCounters,
}

impl RuntimeState {
    fn capture(runtime: &crate::runtime::RelationalRuntime) -> Self {
        Self {
            symbols: runtime.services.symbols.interner_snapshot(),
            configured_symbols: runtime.config().identity.symbol_table.clone(),
            branch_cells: runtime.history().branch_cells_snapshot(),
            catalog: runtime.history().commit_envelopes_snapshot(),
            reference_cost: runtime.phase4_reference_cost_counters(),
            complexity: runtime.performance_access().counters(),
        }
    }

    fn assert_unchanged(&self, runtime: &crate::runtime::RelationalRuntime) {
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
