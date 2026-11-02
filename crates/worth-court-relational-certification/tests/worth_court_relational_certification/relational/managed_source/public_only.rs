//! Real Source staging without a rule, through both installation entrances.
use super::historical_material::{create, observe};
use super::history::{basis, collect, edge, selected};
use super::*;
mod recovery;

#[derive(Clone, Copy, Debug)]
enum Installation {
    Builder,
    Initial,
}

fn install(
    mode: Installation,
    registry: RelationalSchemaRegistry,
    program: ManagedIndexSourceProgram,
) -> (
    RelationalRuntime,
    RelationalInitialSchemaInstallationReceipt,
) {
    let (mut runtime, additions, sources) = match mode {
        Installation::Builder => (
            RelationalRuntimeApi::builder()
                .schema_registry(registry)
                .managed_index_source(program)
                .build()
                .unwrap(),
            RelationalSchemaRegistry::new(),
            vec![],
        ),
        Installation::Initial => (
            RelationalRuntimeApi::builder().build().unwrap(),
            registry,
            vec![program],
        ),
    };
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_managed_sources(additions, vec![], sources)
        .unwrap();
    assert_eq!(
        receipt.custom_invariant_inventory_digest(),
        &custom_invariant_inventory_digest(&[]),
        "no dummy rule"
    );
    (runtime, receipt)
}

fn request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

fn assert_rows(observation: &ManagedIndexSourceObservation, rows: &[EntityId]) {
    assert_eq!(
        observation
            .payload()
            .downcast_ref::<Vec<EntityId>>()
            .unwrap()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        rows.iter().copied().collect::<BTreeSet<_>>(),
        "independent authored membership roster",
    );
}

#[test]
fn public_only_source_builder_and_initial_install_maintain_unopened_scopes_history_and_fork_affinity(
) {
    for mode in [Installation::Builder, Installation::Initial] {
        let (registry, program) = registry();
        let (mut runtime, receipt) = install(mode, registry, program.clone());
        assert!(receipt.has_exact_managed_index_sources(std::slice::from_ref(&program)));
        assert!(!receipt.has_exact_managed_index_sources(&[]));
        let denied = runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install_with_managed_sources(RelationalSchemaRegistry::new(), vec![], vec![])
            .unwrap_err();
        assert_eq!(
            denied.kind(),
            RelationalInitialSchemaInstallationDenialKind::InitialInvariantsAlreadySealed
        );
        let court = Arc::new(owner::Court::default());
        let _registration = runtime
            .publication_companion_port()
            .begin_required_registration()
            .unwrap()
            .activate(Arc::new(owner::SourceOwner(court.clone())))
            .unwrap();
        let host = request();
        let execution = ExecutionRequest::serial(&host);
        let a = create(&runtime, execution, 1, "A");
        let b = create(&runtime, execution, 1, "B");
        let x = create(&runtime, execution, 4, "x");
        let y = create(&runtime, execution, 4, "y");
        assert_rows(&observe(&runtime, execution, &program, a), &[]);
        commit(
            &runtime,
            &execution,
            WorkerIntentBatch::new("public-A-x").push(edge(a, x, "A-x")),
        );
        let original = basis(&runtime);
        court.staged.lock().unwrap().clear();
        // B has never been opened. The shared row must still reach A and B.
        let shared = commit(
            &runtime,
            &execution,
            WorkerIntentBatch::new("public-unopened-B-x").push(edge(b, x, "B-x")),
        );
        assert_eq!(*court.staged.lock().unwrap(), BTreeSet::from([a, b]));
        let a_read = observe(&runtime, execution, &program, a);
        assert_rows(&a_read, &[x]);
        assert_rows(&observe(&runtime, execution, &program, b), &[x]);
        let selector = runtime
            .authored_managed_source_selector(execution, &program, b)
            .unwrap();
        let cursor = runtime
            .authored_managed_source_history_cursor(
                execution,
                &original,
                selected(&runtime, &basis(&runtime)),
                &[selector],
            )
            .unwrap();
        assert_eq!(
            collect(&runtime, execution, cursor),
            [shared.commit.commit_id]
        );
        let branch = BranchId(format!("public-only-{mode:?}"));
        let (_, source) = runtime
            .observe_fork_source(&BranchId("main".into()))
            .unwrap();
        runtime
            .fork_branch(&execution, branch.clone(), source)
            .unwrap();
        let identity = runtime.branch_identity(&branch).unwrap();
        let (_, branch_basis) = runtime.observe_branch(&identity).unwrap();
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&branch_basis.observation())
            .unwrap();
        let fork_read = runtime
            .read_truth()
            .observe_managed_index_source(&execution, &snapshot, &program, a)
            .unwrap();
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
        assert_rows(&fork_read, &[x]);
        assert!(
            !a_read.same_source_scope(&fork_read),
            "fork has actual distinct affinity"
        );
        let mut transaction = runtime
            .begin_branch_transaction(
                &branch_basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("public-fork-A-y").push(edge(a, y, "A-y")),
                execution.allocation_policy(),
            )
            .unwrap();
        transaction.commit(&execution, &runtime).unwrap();
        let (_, current_basis) = runtime.observe_branch(&identity).unwrap();
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&current_basis.observation())
            .unwrap();
        let fork_changed = runtime
            .read_truth()
            .observe_managed_index_source(&execution, &snapshot, &program, a)
            .unwrap();
        assert_rows(&fork_changed, &[x, y]);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
        assert_rows(&observe(&runtime, execution, &program, a), &[x]);
    }
}

#[test]
fn public_only_source_refuses_foreign_program_duplicate_and_atomic_initial_installation() {
    let (registry, program) = registry();
    let (foreign, foreign_program) = super::registry();
    assert_eq!(program.scope_kind(), foreign_program.scope_kind());
    assert_ne!(program, foreign_program);
    assert!(matches!(
        RelationalRuntimeApi::builder()
            .schema_registry(foreign)
            .managed_index_source(program.clone())
            .build(),
        Err(RelationalRuntimeConstructionDenial::ManagedIndexSource(
            DerivedRelationDenial::ForeignDefinition
        ))
    ));
    assert!(matches!(
        RelationalRuntimeApi::builder()
            .schema_registry(registry.clone())
            .managed_index_source(program.clone())
            .managed_index_source(program.clone())
            .build(),
        Err(RelationalRuntimeConstructionDenial::ManagedIndexSource(
            DerivedRelationDenial::DuplicateDefinition(_)
        ))
    ));
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    let denied = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_managed_sources(
            RelationalSchemaRegistry::new(),
            vec![],
            vec![foreign_program],
        )
        .unwrap_err();
    assert_eq!(
        denied.kind(),
        RelationalInitialSchemaInstallationDenialKind::ManagedIndexSourceRejected
    );
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_managed_sources(
            RelationalSchemaRegistry::new(),
            vec![],
            vec![program.clone()],
        )
        .unwrap();
    assert!(receipt.has_exact_managed_index_sources(&[program]));
}
