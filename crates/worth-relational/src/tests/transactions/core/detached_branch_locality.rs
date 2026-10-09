use crate::facade::history::BranchId;
use crate::facade::symbols::{ClientKey, ClientKeySymbolPolicy};
use crate::facade::transactions::{
    ConflictClass, CreatedEntityRef, EntityMutationIntent, EntityReference, MutationIntent,
    RecordRef, RelationSpec, UpdateEntityFieldsIntent,
};
use crate::tests::support::*;

#[test]
fn sibling_target_denial_precedes_raw_client_key_normalization_and_leaves_zero_residue() {
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .client_key_symbol_policy(ClientKeySymbolPolicy::RequireInterned)
        .build();
    create_entity(&runtime, "shared-root");
    fork_from_main(&mut runtime, "storm");
    fork_from_main(&mut runtime, "maintenance");

    let mut storm = begin_on(&runtime, "storm");
    storm
        .push_batch(
            batch_create("storm-exclusive"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let storm_commit = storm
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("storm create commits");
    let storm_only = storm_commit
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(entity_id) => Some(*entity_id),
            RecordRef::Relation(_) => None,
        })
        .expect("storm commit created one entity");

    let mut maintenance = begin_on(&runtime, "maintenance");
    maintenance
        .push_batch(
            batch_create("must-not-be-interned"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    maintenance
        .push_batch(
            WorkerIntentBatch::new("sibling-target").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: storm_only,
                    fields: name_field_patch("must-not-apply"),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let before = RuntimeState::capture(&runtime);

    let error = maintenance
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect_err("a sibling-only target is outside the admitted branch root");

    assert!(matches!(
        error,
        crate::facade::transactions::TransactionCommitError::Conflict { error, .. }
            if matches!(error.class, ConflictClass::StaleTarget { .. })
    ));
    before.assert_unchanged(&runtime);
}

#[test]
fn unowned_created_endpoint_denial_precedes_normalization_and_leaves_zero_residue() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .client_key_symbol_policy(ClientKeySymbolPolicy::RequireInterned)
        .build();
    let missing = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw("unowned-created-endpoint"),
    };
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("unowned-created-endpoint").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw("must-not-be-interned-edge"),
                    source: EntityReference::Created(missing.clone()),
                    target: EntityReference::Created(missing),
                    fields: AspectFieldPatch::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let before = RuntimeState::capture(&runtime);

    let error = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect_err("created endpoints must belong to the same transaction");

    assert!(matches!(
        error,
        crate::facade::transactions::TransactionCommitError::Conflict { error, .. }
            if matches!(error.class, ConflictClass::InvalidRelationEndpoint { .. })
    ));
    before.assert_unchanged(&runtime);
}

fn fork_from_main(runtime: &mut crate::facade::runtime::RelationalRuntime, branch: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(BranchId(branch.into()), source)
        .expect("branch fork succeeds");
}

fn begin_on(
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &str,
) -> crate::facade::mvcc::BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(&BranchId(branch.into()))
        .expect("branch identity exists");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("branch basis is admitted");
    runtime
        .begin_branch_transaction(
            &basis,
            crate::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
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
