//! Real Native deletion and retained cache custody; no projected-row READ claim.
use super::*;
use crate::branch::RelationalBranchDeletionOutcome;
use crate::history::data::BranchId;
use crate::mvcc::ManagedIndexSourceForkPreparation;

#[derive(Debug)]
struct ForkOwner(Owner);
impl RelationalPublicationCompanion for ForkOwner {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        self.0.prepare(context)
    }
    fn prepare_managed_source_fork(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        target: &ManagedIndexSourceForkPreparation<'_>,
    ) -> Result<Box<dyn Any + Send + Sync>, CompanionPreflightStop> {
        crate::authority::commit::pipeline::consult_stage_control(
            execution,
            crate::authority::commit::pipeline::CommitStage::Publication,
        )
        .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?;
        // This court takes only the successful fork. Its actual admitted slot
        // is explicitly relinquished after the real Native deletion below.
        self.0.slots.lock().unwrap().push(target.lookup_slot());
        Ok(Box::new(()))
    }
}

#[test]
fn managed_cache_retained_affinity_survives_deleted_branch_cell_without_currentness() {
    let (schema, program) = managed_cache_schema();
    let mut runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .unwrap();
    let memory = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let host = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&host);
    let scope = create_entity(&execution, &runtime, "cache-scope");
    let port = runtime.publication_companion_port();
    let core = Arc::new(Mutex::new(Weak::new()));
    let drops = Arc::new(AtomicUsize::new(0));
    let owner = Arc::new(ForkOwner(Owner {
        source: program.clone(),
        scope,
        core: core.clone(),
        port: port.clone(),
        drops: drops.clone(),
        slots: Mutex::new(Vec::new()),
    }));
    let registration = port
        .begin_required_registration()
        .unwrap()
        .activate(owner.clone())
        .unwrap();
    create_entity(&execution, &runtime, "source-publication");
    let main_cell = owner.0.slots.lock().unwrap()[0].admitted().unwrap();
    *core.lock().unwrap() = Arc::downgrade(&main_cell.core);
    let main = runtime.main_branch_identity();
    let main_snapshot = snapshot_for_owner_identity(&runtime, &main);
    let main_observation = runtime
        .read_truth()
        .observe_managed_index_source(&execution, &main_snapshot, &program, scope)
        .unwrap();
    let target = BranchId("retained-source-affinity".into());
    let fork = runtime.fork_port();
    let (_, source) = fork.observe_fork_source(main.branch_id()).unwrap();
    fork.fork_branch(&execution, target.clone(), source)
        .unwrap();
    drop(fork);
    let identity = runtime.branch_identity(&target).unwrap();
    let snapshot = snapshot_for_owner_identity(&runtime, &identity);
    let cell = owner.0.slots.lock().unwrap()[1].admitted().unwrap();
    let weak = Arc::downgrade(&cell.core);
    let fork_grant = Arc::downgrade(cell.core._fork_memory.as_ref().unwrap());
    let generation = cell.core.registration_generation;
    let old = runtime
        .read_truth()
        .observe_managed_index_source(&execution, &snapshot, &program, scope)
        .unwrap();
    assert!(
        !old.same_source(&main_observation),
        "fork owns a fresh affinity even when material is shared"
    );
    let selected = old.selected.clone();
    assert!(ManagedSourceReadConditions::from_observation(
        &old,
        runtime.runtime_instance_id(),
        &target,
        selected.root_id(),
        selected.commit_id()
    )
    .is_ok());
    assert!(matches!(
        runtime.delete_branch(&execution, &identity).unwrap(),
        RelationalBranchDeletionOutcome::Deleted(_)
    ));
    let removed = owner.0.slots.lock().unwrap().remove(1);
    drop((removed, cell));
    assert!(
        weak.upgrade().is_none(),
        "retained root and observation must not own the live cell/newer current cache"
    );

    let again = runtime
        .read_truth()
        .observe_managed_index_source(&execution, &snapshot, &program, scope)
        .unwrap();
    assert!(old.same_source(&again));
    assert!(!old.remains_current_at(&selected, generation));
    assert!(matches!(
        ManagedSourceReadConditions::from_observation(
            &old,
            runtime.runtime_instance_id(),
            &target,
            selected.root_id(),
            selected.commit_id()
        ),
        Err(CompanionPreflightStop::ForeignCell)
    ));
    let image = crate::visibility::snapshot_states::resolve_snapshot_basis(&runtime, &snapshot)
        .unwrap()
        .root()
        .managed_source_image
        .get()
        .unwrap()
        .clone();
    assert!(matches!(
        image.admit_selected(&selected, generation + 1),
        Err(Denial::Publication(
            CompanionPreflightStop::RegistrationChanged
        ))
    ));
    assert!(matches!(
        image.affinity.admit(&main_observation.selected, generation),
        Err(Denial::ForeignSource)
    ));
    let foreign = crate::tests::support::runtime_with_test_schema();
    let foreign_snapshot = snapshot_for_owner_identity(&foreign, &foreign.main_branch_identity());
    let foreign_selected = foreign
        .read_truth()
        .positioned_snapshot(&foreign_snapshot)
        .unwrap();
    assert!(matches!(
        image.affinity.admit(&foreign_selected, generation),
        Err(Denial::ForeignSource)
    ));
    drop(foreign);

    let retirement = runtime
        .read_truth()
        .evict_managed_index_source(&execution, &snapshot, &program, scope)
        .unwrap();
    assert!(matches!(
        runtime
            .read_truth()
            .observe_managed_index_source(&execution, &snapshot, &program, scope),
        Err(Denial::RootUnavailable)
    ));
    let loan = runtime
        .read_truth()
        .prepare_managed_index_rebuild(&execution, &snapshot, &program, scope)
        .unwrap();
    let payload: Arc<dyn Any + Send + Sync> = Arc::new(123_u64);
    let (refilled, refill_retirement) = runtime
        .read_truth()
        .finish_managed_index_rebuild(&execution, &snapshot, &loan, &payload)
        .unwrap();
    assert!(
        old.same_source(&refilled),
        "refill preserves actual Source association"
    );
    assert_eq!(refilled.payload().downcast_ref::<u64>(), Some(&123));
    let final_read = runtime
        .read_truth()
        .observe_managed_index_source(&execution, &snapshot, &program, scope)
        .unwrap();
    assert!(refilled.same_source(&final_read));
    assert_eq!(final_read.payload().downcast_ref::<u64>(), Some(&123));
    assert!(!refilled.remains_current_at(&selected, generation));
    assert!(matches!(
        ManagedSourceReadConditions::from_observation(
            &refilled,
            runtime.runtime_instance_id(),
            &target,
            selected.root_id(),
            selected.commit_id()
        ),
        Err(CompanionPreflightStop::ForeignCell)
    ));
    drop((
        old,
        again,
        final_read,
        loan,
        retirement,
        refill_retirement,
        image,
        main_observation,
    ));
    // The original Probe's last Drop runs while the main cell is still live,
    // and verifies that the completed Native registration/current locks are free.
    let retired_main = runtime
        .read_truth()
        .evict_managed_index_source(&execution, &main_snapshot, &program, scope)
        .unwrap();
    drop(retired_main);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    drop((registration, owner, main_cell, runtime));
    assert!(
        fork_grant.upgrade().is_some(),
        "the retained observation's weak Core allocation stays funded"
    );
    drop(refilled);
    assert!(fork_grant.upgrade().is_none());
    assert_eq!(
        memory.charged_memory_bytes(),
        0,
        "all actual affinity/fork/snapshot owners released their grants"
    );
}
