pub use {crate::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError, crate::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram};
pub use {std::collections::BTreeMap};
pub use {worth_relational::facade::config::PublicationConfig};
pub use {worth_relational::facade::runtime::CustomInvariantRegistration};
pub use {worth_relational::facade::runtime::InvariantCatalog, worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial, worth_relational::facade::runtime::RelationalRuntime, worth_relational::facade::runtime::RelationalRuntimeApi};
pub use {worth_relational::facade::transactions::BulkEntityCreateIntent, worth_relational::facade::transactions::BulkRelationCreateIntent, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch};

pub fn main_basis(
    runtime: &RelationalRuntime,
) -> worth_relational::facade::branch::AdmittedRelationalBranchBasis {
    let identity = runtime.main_branch_identity();
    runtime
        .admit_branch_basis(&identity)
        .expect("configured main branch must remain owner-admissible")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupplyChainCompilationError {
    RuntimeConstruction(worth_relational::facade::runtime::RelationalRuntimeConstructionDenial),
    SchemaInstallation(RelationalInitialSchemaInstallationDenial),
    Transaction(TransactionCommitError),
    HandleBinding(HandleBindingError),
    BranchBasis(worth_relational::facade::branch::RelationalBranchBasisDenial),
    SnapshotAdmission(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
}

pub const SUPPLY_CHAIN_BASELINE_PATCH_BUDGET: usize = 16_384;

pub fn compile_supply_chain_baseline(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: CompiledSupplyChainProgram,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_budget(
        execution,
        program,
        SUPPLY_CHAIN_BASELINE_PATCH_BUDGET,
    )
}

pub fn compile_supply_chain_baseline_with_budget(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits(execution, program, max_patch_records_per_commit)
}

pub fn compile_supply_chain_baseline_with_limits(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_custom_invariant(
        execution,
        program,
        max_patch_records_per_commit,
        None,
    )
}

pub fn compile_supply_chain_baseline_with_custom_invariant(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: CompiledSupplyChainProgram,
    custom_invariant: CustomInvariantRegistration,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_custom_invariant(
        execution,
        program,
        SUPPLY_CHAIN_BASELINE_PATCH_BUDGET,
        Some(custom_invariant),
    )
}

pub fn compile_supply_chain_baseline_with_invariant_catalog(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: CompiledSupplyChainProgram,
    invariant_catalog: InvariantCatalog,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariant(
        execution,
        program,
        SUPPLY_CHAIN_BASELINE_PATCH_BUDGET,
        Some(invariant_catalog),
        None,
    )
}

pub fn compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
    invariant_catalog: InvariantCatalog,
    custom_invariants: Vec<CustomInvariantRegistration>,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariants(
        execution,
        program,
        max_patch_records_per_commit,
        Some(invariant_catalog),
        custom_invariants,
    )
}

pub fn compile_supply_chain_baseline_with_limits_and_custom_invariant(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
    custom_invariant: Option<CustomInvariantRegistration>,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariant(
        execution,
        program,
        max_patch_records_per_commit,
        None,
        custom_invariant,
    )
}

pub fn compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariant(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
    invariant_catalog: Option<InvariantCatalog>,
    custom_invariant: Option<CustomInvariantRegistration>,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariants(
        execution,
        program,
        max_patch_records_per_commit,
        invariant_catalog,
        custom_invariant.into_iter().collect(),
    )
}

pub fn compile_supply_chain_baseline_with_limits_and_catalog_and_custom_invariants(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: CompiledSupplyChainProgram,
    max_patch_records_per_commit: usize,
    invariant_catalog: Option<InvariantCatalog>,
    custom_invariants: Vec<CustomInvariantRegistration>,
) -> Result<ProductionSeededSupplyChainWorld, SupplyChainCompilationError> {
    let mut builder = worth_relational::facade::runtime::RelationalRuntimeApi::builder().publication(PublicationConfig {
        coherent_publication_required: true,
        max_patch_records_per_commit: Some(max_patch_records_per_commit),
        max_published_snapshot_handles: Some(256),
        max_active_snapshot_handles: Some(4_096),
        max_transaction_savepoints: 4_096,
        max_prepared_candidates: Some(1_024),
        candidate_max_lifetime_millis: 30_000,
    });
    if let Some(invariant_catalog) = invariant_catalog {
        builder = builder.invariant_catalog(invariant_catalog);
    }
    for custom_invariant in custom_invariants {
        builder = builder.custom_invariant(custom_invariant);
    }
    let mut runtime = builder
        .build()
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError::RuntimeConstruction)?;
    let schema_receipt = runtime
        .prepare_initial_schema_installation()
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError::SchemaInstallation)?
        .install(program.schema_registry().clone())
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError::SchemaInstallation)?;

    let commit_result = commit_definition(execution, &runtime, &program)?;
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime
        .observe_branch(&identity)
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError::BranchBasis)?;
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError::SnapshotAdmission)?;
    let handles = crate::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles::bind(&program, &commit_result, snapshot)
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError::HandleBinding)?;

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

pub fn commit_definition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
                worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-empty-baseline"),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("empty baseline staging fits the configured transaction budget");
        return transaction
            .commit(execution, runtime)
            .map_err(transaction_error);
    }

    let mut batch = worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-baseline");
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
        .commit(execution, runtime)
        .map_err(transaction_error)
}

pub fn bulk_entity_intents(program: &CompiledSupplyChainProgram) -> Vec<MutationIntent> {
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
        .map(|intent| worth_relational::facade::transactions::MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::BulkEntities(intent)))
        .collect()
}

pub fn bulk_relation_intents(program: &CompiledSupplyChainProgram) -> Vec<MutationIntent> {
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
        .map(|intent| worth_relational::facade::transactions::MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::BulkRelations(intent)))
        .collect()
}

pub fn transaction_error(error: TransactionCommitError) -> SupplyChainCompilationError {
    crate::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError::Transaction(error)
}