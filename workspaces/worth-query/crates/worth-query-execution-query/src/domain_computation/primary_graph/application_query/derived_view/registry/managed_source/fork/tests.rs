//! Real Native cancellation exercises the production Query lookup guard.
//! Empty material proves lookup custody only; geometry has its public court.
use super::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    mvcc::*, runtime::RelationalRuntimeApi, schema::*, transactions::*,
};

struct Companion {
    registry: Arc<ManagedDerivedViewRegistry>,
    source_cells: Mutex<Vec<CompanionBranchCellSlot<ManagedIndexBranchImage>>>,
    cancel: Mutex<Option<CancellationSource>>,
    cancel_after_reserve: Mutex<Option<CancellationSource>>,
    use_reserve_source_image: bool,
    memory: SerialMemoryBudget,
    staged_bytes: AtomicU64,
}
impl std::fmt::Debug for Companion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ActualQueryForkLookupCourt")
    }
}
impl RelationalPublicationCompanion for Companion {
    fn prepare_managed_source_placement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        target: &ManagedIndexSourceCellPlacement<'_>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, Stop> {
        let lookup = self.registry.prepare_source_placement(execution, target)?;
        let state = self.registry.state.lock().unwrap();
        assert!(state.source_branches[target.branch()]
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
    fn prepare_managed_source(
        &self,
        _: &ExecutionRequest<'_, '_>,
        source: &ManagedIndexSourcePreparation<'_>,
    ) -> Result<Vec<PreparedManagedIndexSourceRoot>, Denial> {
        refill_support::collect_scope_roots(source)
    }
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, Stop> {
        let mark = context
            .mint_selected_branch_cell(Arc::new(()))?
            .reserve_preflight(context)?;
        let source = if self.use_reserve_source_image {
            let reserved = self
                .registry
                .reserve_source_image(context)?
                .expect("public cache owner");
            let state = self.registry.state.lock().unwrap();
            let entry = &state.source_branches[context.branch_id()];
            assert!(entry.slot.admitted().is_none() && !entry.slot.is_expired_pending());
            drop(state);
            if let Some(cancel) = self.cancel_after_reserve.lock().unwrap().take() {
                cancel.cancel();
            }
            reserved
        } else {
            let source =
                context.mint_selected_branch_cell(Arc::new(ManagedIndexBranchImage::default()))?;
            self.source_cells.lock().unwrap().push(source.lookup_slot());
            source.reserve_preflight(context)?
        };
        let next = context.prepare_managed_source_image(&source)?;
        context.seal_managed_source_replacement(mark, Arc::new(()), source, next)
    }
}

fn req(mem: SerialMemoryBudget, cancel: worth_execution::CancellationToken) -> SerialRequest {
    SerialRequest::from_memory(mem, cancel, None)
}

#[test]
fn managed_fork_cancelled_lookup_releases_the_actual_registry_entry() {
    let schema = refill_support::anchor_schema();
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
        cancel_after_reserve: Mutex::new(None),
        use_reserve_source_image: false,
        memory: memory.clone(),
        staged_bytes: AtomicU64::new(0),
    });
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(owner.clone())
        .unwrap();
    let startup = req(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
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
            refill_support::anchor_batch("source-anchor"),
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
            refill_support::anchor_batch("foreign-anchor"),
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

#[path = "refill_support.rs"]
mod refill_support;

#[test]
fn refused_first_post_recovery_publish_leaves_expired_slot_that_cold_refill_replaces() {
    use crate::domain_computation::primary_graph::application_query::managed_index::ManagedSourceInstallation;
    use refill_support::{refill_schema, refused_publish_batch, seed_batch};
    use worth_relational::facade::runtime::ManagedIndexSourceProgram;

    let (reg, program) = refill_schema();
    let install = |r: RelationalSchemaRegistry, p: ManagedIndexSourceProgram| {
        let mut rt = RelationalRuntimeApi::builder()
            .schema_registry(r)
            .managed_index_source(p)
            .build()
            .unwrap();
        let rc = rt
            .prepare_initial_schema_installation()
            .unwrap()
            .install_with_managed_sources(RelationalSchemaRegistry::new(), vec![], vec![])
            .unwrap();
        (rt, rc)
    };
    let companion = |registry: &Arc<ManagedDerivedViewRegistry>, cancel_after| {
        Arc::new(Companion {
            registry: Arc::clone(registry),
            source_cells: Mutex::new(Vec::new()),
            cancel: Mutex::new(None),
            cancel_after_reserve: Mutex::new(cancel_after),
            use_reserve_source_image: true,
            memory: SerialMemoryBudget::new(None),
            staged_bytes: AtomicU64::new(0),
        })
    };
    let (seed_rt, receipt) = install(reg.clone(), program.clone());
    let seed_reg = Arc::new(ManagedDerivedViewRegistry::with_sources(
        ManagedSourceInstallation::with_public_cache_owner(),
        &seed_rt.main_branch_identity(),
    ));
    let _s = seed_rt
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(companion(&seed_reg, None))
        .unwrap();
    let unbounded = req(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
    );
    let seed_exec = ExecutionRequest::serial(&unbounded);
    let (ca, cx, batch) = seed_batch();
    let (_, basis) = seed_rt
        .observe_branch(&seed_rt.main_branch_identity())
        .unwrap();
    let mut tx = seed_rt
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(batch, seed_exec.allocation_policy()).unwrap();
    let born = tx.commit(&seed_exec, &seed_rt).unwrap();
    let (a, x) = (
        born.created_entity(&ca).unwrap(),
        born.created_entity(&cx).unwrap(),
    );
    let (ckpt, members) =
        worth_testkit_relational::cold_recovery::capture_external_checkpoint(&seed_exec, &seed_rt);

    let (mut recovered, _) = install(reg, program.clone());
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&seed_exec, &ckpt, members)
        .unwrap();
    recovered
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let cancel_first = CancellationSource::new();
    let registry = Arc::new(ManagedDerivedViewRegistry::with_sources(
        ManagedSourceInstallation::with_public_cache_owner(),
        &recovered.main_branch_identity(),
    ));
    let reg_guard = recovered
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(companion(&registry, Some(cancel_first.clone())))
        .unwrap();
    let main = BranchId("main".into());
    let mem_commit = SerialMemoryBudget::new(Some(64 << 20));
    let mem_refill = SerialMemoryBudget::new(Some(64 << 20));
    let c_req = req(mem_commit.clone(), cancel_first.token());
    let r_req = req(mem_refill.clone(), CancellationSource::new().token());
    let cancelled_exec = ExecutionRequest::serial(&c_req);
    let execution = ExecutionRequest::serial(&r_req);
    let (_, head_basis) = recovered
        .observe_branch(&recovered.main_branch_identity())
        .unwrap();
    let snap = recovered
        .snapshots()
        .snapshot_for_observation(&head_basis.observation())
        .unwrap();

    // First post-recovery publish reserves a pending slot then cancels, leaving an expired slot.
    let mut pending_tx = recovered
        .begin_branch_transaction(&head_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    pending_tx
        .push_batch(
            refused_publish_batch(),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let stop = pending_tx.commit(&cancelled_exec, &recovered).unwrap_err();
    assert!(matches!(
        stop,
        TransactionCommitError::Execution { .. } | TransactionCommitError::Interrupted { .. }
    ));
    drop(stop);
    let before_refill_bytes = mem_commit.charged_memory_bytes();
    let expired_alloc =
        Arc::downgrade(&registry.state.lock().unwrap().source_branches[&main].allocation);
    assert!(registry.state.lock().unwrap().source_branches[&main]
        .slot
        .is_expired_pending());

    // Explicit cold rebuild at the exact same head replaces the expired slot and retires it.
    let loan = recovered
        .read_truth()
        .prepare_managed_index_rebuild(&execution, &snap, &program, a)
        .expect("cold refill must replace an expired pending slot at the genuine cold head");
    assert!(expired_alloc.upgrade().is_none());
    assert!(mem_commit.charged_memory_bytes() < before_refill_bytes);
    assert!(registry.state.lock().unwrap().source_branches[&main]
        .slot
        .admitted()
        .is_some());
    let rows = loan.rows(seed_exec);
    let read = rows.acquire_all().unwrap();
    let members: Vec<_> = rows
        .occurrences(&read)
        .unwrap()
        .iter()
        .map(|o| o.entity())
        .collect();
    assert_eq!(members, vec![x]);
    let payload: Arc<dyn std::any::Any + Send + Sync> = Arc::new(members);
    let (refilled, retirement) = recovered
        .read_truth()
        .finish_managed_index_rebuild(&execution, &snap, &loan, &payload)
        .unwrap();
    drop(retirement);
    let ready = recovered
        .read_truth()
        .observe_managed_index_source_if_ready(&execution, &snap, &program, a)
        .unwrap()
        .unwrap();
    assert!(refilled.same_source(&ready));
    recovered.snapshots().release_snapshot(&snap).unwrap();
    drop((
        read, loan, refilled, ready, head_basis, reg_guard, recovered, registry,
    ));
    assert_eq!(mem_commit.charged_memory_bytes(), 0);
    assert_eq!(mem_refill.charged_memory_bytes(), 0);
}
