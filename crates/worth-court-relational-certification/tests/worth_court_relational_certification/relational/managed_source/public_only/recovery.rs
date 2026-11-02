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
