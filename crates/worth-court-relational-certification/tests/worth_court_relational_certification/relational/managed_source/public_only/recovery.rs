//! A real checkpoint restore must retain registration and continue maintenance.
use super::super::history::{edge_id, endpoints};
use super::*;

#[test]
fn public_only_checkpoint_readmission_preserves_roster_and_post_restore_source_staging() {
    let (registry, program) = registry();
    let (runtime, receipt) = install(Installation::Builder, registry.clone(), program.clone());
    let court = Arc::new(owner::Court::default());
    let _registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court)))
        .unwrap();
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let a = create(&runtime, execution, 1, "A");
    let x = create(&runtime, execution, 4, "x");
    let y = create(&runtime, execution, 4, "y");
    let linked = commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("before-checkpoint-A-x").push(edge(a, x, "A-x")),
    );
    let relation = edge_id(&linked, a, x, "A-x");
    assert_rows(&observe(&runtime, execution, &program, a), &[x]);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();

    let mut missing = RelationalRuntimeApi::builder()
        .schema_registry(registry.clone())
        .build()
        .unwrap();
    missing
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    missing
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    let denied = missing
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .unwrap_err();
    assert_eq!(
        denied.kind(),
        RelationalInitialSchemaInstallationDenialKind::RecoveredAuthorityMismatch
    );

    let (mut recovered, _) = install(Installation::Builder, registry, program.clone());
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    let readmitted = recovered
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert!(readmitted.has_exact_managed_index_sources(std::slice::from_ref(&program)));
    let court = Arc::new(owner::Court::default());
    let _registration = recovered
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court.clone())))
        .unwrap();
    let original = basis(&recovered);
    let changed = endpoints(&recovered, execution, relation, a, y);
    assert_eq!(*court.staged.lock().unwrap(), BTreeSet::from([a]));
    let observed = observe(&recovered, execution, &program, a);
    assert_rows(&observed, &[y]);
    assert_eq!(observed.source_revision(), changed.commit.commit_id);
    let selector = recovered
        .authored_managed_source_selector(execution, &program, a)
        .unwrap();
    // Actual Source staging does not reconstruct the absent parent history
    // generation. Native preserves its explicit cold-accelerator refusal.
    assert!(matches!(
        recovered.authored_managed_source_history_cursor(
            execution,
            &original,
            selected(&recovered, &basis(&recovered)),
            &[selector],
        ),
        Err(
            worth_relational::facade::history::RelationalAuthoredWriteDenial::GenerationUnavailable
        )
    ));
}

fn snapshot_at_main(
    runtime: &RelationalRuntime,
) -> worth_relational::facade::snapshots::SnapshotHandle {
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&current.observation())
        .unwrap()
}

#[test]
fn readmitted_checkpoint_cold_source_opens_empty_and_refills_live_cell() {
    let (registry, program) = registry();
    let (_, foreign_program) = super::super::registry();
    let (runtime, receipt) = install(Installation::Builder, registry.clone(), program.clone());
    let court = Arc::new(owner::Court::default());
    let _registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court)))
        .unwrap();
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let a = create(&runtime, execution, 1, "A");
    let x = create(&runtime, execution, 4, "x");
    let y = create(&runtime, execution, 4, "y");
    let linked = commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("cold-before-checkpoint-A-x").push(edge(a, x, "A-x")),
    );
    let relation = edge_id(&linked, a, x, "A-x");
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();

    let (mut recovered, _) = install(Installation::Builder, registry.clone(), program.clone());
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    recovered
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .unwrap();
    let court = Arc::new(owner::Court::default());
    let _registration = recovered
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court.clone())))
        .unwrap();
    let restored_snapshot = snapshot_at_main(&recovered);
    assert!(
        recovered
            .read_truth()
            .observe_managed_index_source_if_ready(&execution, &restored_snapshot, &program, a)
            .unwrap()
            .is_none(),
        "cold readmitted root opens empty for explicit Created lifecycle"
    );
    assert!(matches!(
        recovered.read_truth().observe_managed_index_source(
            &execution,
            &restored_snapshot,
            &program,
            a
        ),
        Err(ManagedIndexSourcePreparationDenial::RootUnavailable)
    ));
    // Foreign program and wrong-kind scope refuse before attaching a cell.
    assert!(matches!(
        recovered.read_truth().prepare_managed_index_rebuild(
            &execution,
            &restored_snapshot,
            &foreign_program,
            a,
        ),
        Err(ManagedIndexSourcePreparationDenial::ForeignSource)
    ));
    assert!(matches!(
        recovered.read_truth().prepare_managed_index_rebuild(
            &execution,
            &restored_snapshot,
            &program,
            x,
        ),
        Err(ManagedIndexSourcePreparationDenial::ForeignSource)
    ));
    assert!(
        court.source_cells.lock().unwrap().is_empty(),
        "refused cold rebuild must not attach a companion cell"
    );

    // First valid rebuild attaches a live cell and prepares exact cold rows.
    let loan = recovered
        .read_truth()
        .prepare_managed_index_rebuild(&execution, &restored_snapshot, &program, a)
        .unwrap();
    let competing = recovered
        .read_truth()
        .prepare_managed_index_rebuild(&execution, &restored_snapshot, &program, a)
        .unwrap();
    assert!(
        court
            .source_cells
            .lock()
            .unwrap()
            .get(&BranchId("main".into()))
            .and_then(|slot| slot.admitted())
            .is_some(),
        "cold rebuild placement admits the branch cell for subsequent live writes"
    );
    let rows = loan.rows(execution);
    let read = rows.acquire_all().unwrap();
    let members: Vec<EntityId> = rows
        .occurrences(&read)
        .unwrap()
        .iter()
        .map(|occurrence| occurrence.entity())
        .collect();
    assert_eq!(members, vec![x]);
    let payload: Arc<dyn std::any::Any + Send + Sync> = Arc::new(members);
    let (refilled, retirement) = recovered
        .read_truth()
        .finish_managed_index_rebuild(&execution, &restored_snapshot, &loan, &payload)
        .unwrap();
    drop(retirement);
    assert_rows(&refilled, &[x]);
    assert!(matches!(
        recovered.read_truth().finish_managed_index_rebuild(
            &execution,
            &restored_snapshot,
            &competing,
            &payload,
        ),
        Err(ManagedIndexSourcePreparationDenial::Publication(
            worth_relational::facade::mvcc::CompanionPreflightStop::ManagedSourceChanged
        ))
    ));
    assert!(recovered
        .read_truth()
        .managed_index_source_is_current(&execution, &restored_snapshot, &refilled)
        .unwrap());
    let ready = recovered
        .read_truth()
        .observe_managed_index_source_if_ready(&execution, &restored_snapshot, &program, a)
        .unwrap()
        .expect("refilled cold source is immediately ready");
    assert!(refilled.same_source(&ready));
    assert_rows(&ready, &[x]);

    // Subsequent live write reuses the placed cell and advances the source.
    let changed = endpoints(&recovered, execution, relation, a, y);
    assert_eq!(*court.staged.lock().unwrap(), BTreeSet::from([a]));
    let new_snapshot = snapshot_at_main(&recovered);
    assert!(!recovered
        .read_truth()
        .managed_index_source_is_current(&execution, &new_snapshot, &refilled)
        .unwrap());
    let advanced = observe(&recovered, execution, &program, a);
    assert_rows(&advanced, &[y]);
    assert_eq!(advanced.source_revision(), changed.commit.commit_id);
    recovered
        .snapshots()
        .release_snapshot(&restored_snapshot)
        .unwrap();
    recovered
        .snapshots()
        .release_snapshot(&new_snapshot)
        .unwrap();

    // A stale readmitted snapshot cannot attach a cell after the branch moved on.
    let (mut stale_runtime, _) = install(Installation::Builder, registry, program.clone());
    stale_runtime
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    stale_runtime
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let stale_court = Arc::new(owner::Court::default());
    let _stale_reg = stale_runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(stale_court)))
        .unwrap();
    let stale_snapshot = snapshot_at_main(&stale_runtime);
    endpoints(&stale_runtime, execution, relation, a, y);
    assert!(matches!(
        stale_runtime.read_truth().prepare_managed_index_rebuild(
            &execution,
            &stale_snapshot,
            &program,
            a,
        ),
        Err(ManagedIndexSourcePreparationDenial::ForeignSource)
    ));
    stale_runtime
        .snapshots()
        .release_snapshot(&stale_snapshot)
        .unwrap();
}
