use super::handles::{HandleBindingError, SupplyChainSemanticHandles};
use super::production_world::ProductionSeededSupplyChainWorld;
use super::program::CompiledSupplyChainProgram;
use std::collections::BTreeMap;
use worth_relational::facade::config::PublicationConfig;
use worth_relational::facade::runtime::CustomInvariantRegistration;
use worth_relational::facade::runtime::{
    InvariantCatalog, RelationalInitialSchemaInstallationDenial, RelationalRuntime,
    RelationalRuntimeApi,
};
use worth_relational::facade::transactions::{
    BulkEntityCreateIntent, BulkRelationCreateIntent, CreateIntent, MutationIntent,
    TransactionCommitError, WorkerIntentBatch,
};

fn main_basis(
    runtime: &RelationalRuntime,
) -> worth_relational::facade::branch::AdmittedRelationalBranchBasis {
    let identity = runtime.main_branch_identity();
    runtime
        .admit_branch_basis(&identity)
        .expect("configured main branch must remain owner-admissible")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SupplyChainCompilationError {
    SchemaInstallation(RelationalInitialSchemaInstallationDenial),
    Transaction(TransactionCommitError),
    HandleBinding(HandleBindingError),
    BranchBasis(worth_relational::facade::branch::RelationalBranchBasisDenial),
    SnapshotAdmission(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
}

const SUPPLY_CHAIN_BASELINE_PATCH_BUDGET: usize = 16_384;

pub(crate) fn compile_supply_chain_baseline(
    program: CompiledSupplyChainProgram,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_budget(program, SUPPLY_CHAIN_BASELINE_PATCH_BUDGET)
}

pub(crate) fn compile_supply_chain_baseline_with_budget(
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits(program, max_patch_records_per_commit)
}

pub(crate) fn compile_supply_chain_baseline_with_limits(
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_custom_invariant(
        program,
        max_patch_records_per_commit,
        None,
    )
}

pub(crate) fn compile_supply_chain_baseline_with_custom_invariant(
    program: CompiledSupplyChainProgram,
    custom_invariant: CustomInvariantRegistration,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_custom_invariant(
        program,
        SUPPLY_CHAIN_BASELINE_PATCH_BUDGET,
        Some(custom_invariant),
    )
}

pub(crate) fn compile_supply_chain_baseline_with_invariant_catalog(
    program: CompiledSupplyChainProgram,
    invariant_catalog: InvariantCatalog,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariant(
        program,
        SUPPLY_CHAIN_BASELINE_PATCH_BUDGET,
        Some(invariant_catalog),
        None,
    )
}

pub(crate) fn compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants(
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
    invariant_catalog: InvariantCatalog,
    custom_invariants: Vec<CustomInvariantRegistration>,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariants(
        program,
        max_patch_records_per_commit,
        Some(invariant_catalog),
        custom_invariants,
    )
}

fn compile_supply_chain_baseline_with_limits_and_custom_invariant(
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
    custom_invariant: Option<CustomInvariantRegistration>,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariant(
        program,
        max_patch_records_per_commit,
        None,
        custom_invariant,
    )
}

fn compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariant(
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
    invariant_catalog: Option<InvariantCatalog>,
    custom_invariant: Option<CustomInvariantRegistration>,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariants(
        program,
        max_patch_records_per_commit,
        invariant_catalog,
        custom_invariant.into_iter().collect(),
    )
}

fn compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariants(
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
    invariant_catalog: Option<InvariantCatalog>,
    custom_invariants: Vec<CustomInvariantRegistration>,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    let mut builder = RelationalRuntimeApi::builder().publication(PublicationConfig {
        coherent_publication_required: true,
        max_patch_records_per_commit,
        max_published_snapshot_handles: 256,
        max_active_snapshot_handles: 4_096,
        max_transaction_savepoints: 4_096,
        max_prepared_candidates: 1_024,
        candidate_max_lifetime_millis: 30_000,
        max_prepared_root_bytes: 268_435_456,
    });
    if let Some(invariant_catalog) = invariant_catalog {
        builder = builder.invariant_catalog(invariant_catalog);
    }
    for custom_invariant in custom_invariants {
        builder = builder.custom_invariant(custom_invariant);
    }
    let mut runtime = builder.build();
    let schema_receipt = runtime
        .prepare_initial_schema_installation()
        .map_err(SupplyChainCompilationError::SchemaInstallation)?
        .install(program.schema_registry().clone())
        .map_err(SupplyChainCompilationError::SchemaInstallation)?;

    let commit_result = commit_definition(&runtime, &program)?;
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime
        .observe_branch(&identity)
        .map_err(SupplyChainCompilationError::BranchBasis)?;
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .map_err(SupplyChainCompilationError::SnapshotAdmission)?;
    let handles = SupplyChainSemanticHandles::bind(&program, &commit_result, snapshot)
        .map_err(SupplyChainCompilationError::HandleBinding)?;

    Ok(ProductionSeededSupplyChainWorld {
        runtime,
        basis,
        program,
        handles,
        commit: commit_result.commit.clone(),
        commit_result,
        schema_receipt,
    })
}

fn commit_definition(
    runtime: &RelationalRuntime,
    program: &CompiledSupplyChainProgram,
) -> Result<worth_relational::facade::transactions::CommitResult, SupplyChainCompilationError> {
    if program.entity_specs().is_empty() && program.relation_specs().is_empty() {
        let mut transaction = {
            let basis = main_basis(runtime);
            runtime
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("owner-admitted transaction context")
        };
        transaction
            .push_batch(
                WorkerIntentBatch::new("supply-chain-empty-baseline"),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("empty baseline staging fits the configured transaction budget");
        return transaction
            .commit(
                runtime,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(transaction_error);
    }

    let mut batch = WorkerIntentBatch::new("supply-chain-baseline");
    for intent in bulk_entity_intents(program) {
        batch = batch.push(intent);
    }
    for intent in bulk_relation_intents(program) {
        batch = batch.push(intent);
    }

    let mut transaction = {
        let basis = main_basis(runtime);
        runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("Supply Chain baseline staging fits its configured transaction budget");
    transaction
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(transaction_error)
}

fn bulk_entity_intents(program: &CompiledSupplyChainProgram) -> Vec<MutationIntent> {
    let mut grouped = BTreeMap::<_, BulkEntityCreateIntent>::new();
    for spec in program.all_entity_specs() {
        let entry = grouped
            .entry(spec.kind_id)
            .or_insert_with(|| BulkEntityCreateIntent {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_keys: Vec::new(),
                field_patches: Vec::new(),
            });
        entry.client_keys.push(spec.client_key.clone());
        entry.field_patches.push(spec.fields.clone());
    }
    grouped
        .into_values()
        .map(|intent| MutationIntent::Create(CreateIntent::BulkEntities(intent)))
        .collect()
}

fn bulk_relation_intents(program: &CompiledSupplyChainProgram) -> Vec<MutationIntent> {
    let mut grouped = BTreeMap::<_, BulkRelationCreateIntent>::new();
    for spec in program.all_relation_specs() {
        let entry = grouped
            .entry(spec.kind_id)
            .or_insert_with(|| BulkRelationCreateIntent {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_keys: Vec::new(),
                endpoints: Vec::new(),
                field_patches: Vec::new(),
            });
        entry.client_keys.push(spec.client_key.clone());
        entry
            .endpoints
            .push((spec.source.clone(), spec.target.clone()));
        entry.field_patches.push(spec.fields.clone());
    }
    grouped
        .into_values()
        .map(|intent| MutationIntent::Create(CreateIntent::BulkRelations(intent)))
        .collect()
}

fn transaction_error(error: TransactionCommitError) -> SupplyChainCompilationError {
    SupplyChainCompilationError::Transaction(error)
}
