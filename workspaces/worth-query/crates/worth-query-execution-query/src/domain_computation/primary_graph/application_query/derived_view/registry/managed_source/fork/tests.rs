//! Real Native cancellation exercises the production Query lookup guard.
//! Empty material proves lookup custody only; geometry has its public court.
use super::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::*, mvcc::*, runtime::RelationalRuntimeApi, schema::*, symbols::ClientKey,
    transactions::*,
};

struct Companion {
    registry: Arc<ManagedDerivedViewRegistry>,
    source_cells: Mutex<Vec<CompanionBranchCellSlot<ManagedIndexBranchImage>>>,
    cancel: Mutex<Option<CancellationSource>>,
    memory: SerialMemoryBudget,
    staged_bytes: AtomicU64,
}
impl std::fmt::Debug for Companion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ActualQueryForkLookupCourt")
    }
}
impl RelationalPublicationCompanion for Companion {
    fn prepare_managed_source_fork(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        target: &ManagedIndexSourceForkPreparation<'_>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, Stop> {
        let lookup = self.registry.prepare_source_fork(execution, target)?;
        let state = self.registry.state.lock().unwrap();
        assert!(state.source_branches[target.target_branch()]
            .slot
            .admitted()
            .is_none());
        drop(state);
        self.staged_bytes
            .store(self.memory.charged_memory_bytes(), Ordering::SeqCst);
        if let Some(cancel) = self.cancel.lock().unwrap().take() {
            cancel.cancel();
        }
        Ok(lookup)
    }
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, Stop> {
        let mark = context.mint_selected_branch_cell(Arc::new(()))?;
        let mark = mark.reserve_preflight(context)?;
        let source =
            context.mint_selected_branch_cell(Arc::new(ManagedIndexBranchImage::default()))?;
        self.source_cells.lock().unwrap().push(source.lookup_slot());
        let source = source.reserve_preflight(context)?;
        let next = context.prepare_managed_source_image(&source)?;
        context.seal_managed_source_replacement(mark, Arc::new(()), source, next)
    }
}

#[test]
fn managed_fork_cancelled_lookup_releases_the_actual_registry_entry() {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "anchor".into(),
            schema_id: SchemaId("fork-lookup-court".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema.clone())
        .build()
        .unwrap();
    let registry = Arc::new(ManagedDerivedViewRegistry::with_sources(
        Default::default(),
        &runtime.main_branch_identity(),
    ));
    let memory = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let cancel = CancellationSource::new();
    let owner = Arc::new(Companion {
        registry: Arc::clone(&registry),
        source_cells: Mutex::new(Vec::new()),
        cancel: Mutex::new(Some(cancel.clone())),
        memory: memory.clone(),
        staged_bytes: AtomicU64::new(0),
    });
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(owner.clone())
        .unwrap();
    let startup = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&startup);
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("source-anchor").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("source-anchor"),
                    fields: Default::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let seed = transaction.commit(&execution, &runtime).unwrap();
    let request = SerialRequest::from_memory(memory.clone(), cancel.token(), None);
    let target = BranchId("cancelled-query-lookup".into());
    let (_, source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    let stopped = runtime.fork_branch(&ExecutionRequest::serial(&request), target.clone(), source);
    assert!(
        matches!(
            stopped,
            Err(
                worth_relational::facade::branch::RelationalBranchTransitionStop::Denied(
                    worth_relational::facade::branch::RelationalForkDenial::ManagedSource(
                        Stop::Execution(RelationalExecutionDenialCause::Cancelled)
                    )
                )
            )
        ),
        "{stopped:?}"
    );
    assert!(
        owner.staged_bytes.load(Ordering::SeqCst) > 0,
        "original payer held real pending lookup/header grants before cancellation"
    );
    assert!(runtime.branch_identity(&target).is_err());
    assert!(registry.state.lock().unwrap().source_branches.is_empty());
    assert_eq!(
        memory.charged_memory_bytes(),
        0,
        "cancelled fork removes actual metadata and returns all original grants"
    );
    // A new request retries that same genuine target reservation and installs
    // an admitted association. No old pending wrapper can block or satisfy it.
    let retry = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let (_, source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    runtime
        .fork_branch(&ExecutionRequest::serial(&retry), target.clone(), source)
        .unwrap();
    assert!(registry.state.lock().unwrap().source_branches[&target]
        .slot
        .admitted()
        .is_some());
    assert!(memory.charged_memory_bytes() > 0);
    let allocation =
        Arc::downgrade(&registry.state.lock().unwrap().source_branches[&target].allocation);
    // A genuine deletion from another owner with the same name cannot retire it.
    let mut foreign = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .unwrap();
    let foreign_basis = foreign
        .admit_branch_basis(&foreign.main_branch_identity())
        .unwrap();
    let mut foreign_seed = foreign
        .begin_branch_transaction(&foreign_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    foreign_seed
        .push_batch(
            WorkerIntentBatch::new("foreign-anchor").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("foreign-anchor"),
                    fields: Default::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let _foreign_seed = foreign_seed.commit(&execution, &foreign).unwrap();
    let (_, source) = foreign
        .observe_fork_source(foreign.main_branch_identity().branch_id())
        .unwrap();
    foreign
        .fork_branch(&execution, target.clone(), source)
        .unwrap();
    let foreign_identity = foreign.branch_identity(&target).unwrap();
    let outcome = foreign
        .delete_branch(&execution, &foreign_identity)
        .unwrap();
    registry.retire_source_branch(outcome.deleted().expect("real foreign deletion"));
    assert!(allocation.upgrade().is_some());
    let identity = runtime.branch_identity(&target).unwrap();
    assert!(registry
        .state
        .lock()
        .unwrap()
        .source_branches
        .contains_key(identity.branch_id()));
    let outcome = runtime.delete_branch(&execution, &identity).unwrap();
    registry.retire_source_branch(outcome.deleted().expect("real local deletion"));
    assert!(!registry
        .state
        .lock()
        .unwrap()
        .source_branches
        .contains_key(identity.branch_id()));
    assert!(registry.state.lock().unwrap().source_branches.is_empty());
    assert!(
        allocation.upgrade().is_none(),
        "actual retired Query lookup grant releases immediately"
    );
    drop((basis, seed, runtime, registration, owner, registry));
    assert_eq!(memory.charged_memory_bytes(), 0);
}
