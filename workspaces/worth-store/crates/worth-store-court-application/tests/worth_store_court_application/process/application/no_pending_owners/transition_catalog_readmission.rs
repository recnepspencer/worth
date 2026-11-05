//! Completed original occurrences restore exact target executors on real roots.
use super::{transition_open_original as original, *};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
    HomeProgramTransitionTargetMeaning,
};
use worth_relational::facade::{
    durability::RelationalHomeProgramCatalogReadmissionDenial as Denial,
    mvcc::{RelationalCallerAssociation, RelationalTransactionIntent},
    runtime::RelationalHomeProgramStepContracts,
};
use worth_runtime_world::facade::{RuntimeWorldCancellationSource, WorldOpenAttemptOutcome};

mod publication;
mod request_controls;

#[test]
fn a_completed_transition_occurrence_restores_target_contracts_and_continues_only_its_missing_suffix(
) {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let earlier = Arc::new(AtomicUsize::new(0));
    let original_receipt = publication::target(directory.path(), &execution, earlier.clone());
    assert!(earlier.load(Ordering::SeqCst) > 0);
    let resumed = Arc::new(AtomicUsize::new(0));
    let readmission_memory = worth_execution::SerialMemoryBudget::new(None);
    let readmission_request = worth_execution::SerialRequest::from_memory(
        readmission_memory.clone(),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let readmission_execution = ExecutionRequest::serial(&readmission_request);
    {
        let fixture = original::reopen(directory.path(), &execution);
        let source = fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap();
        assert_eq!(source.reference_generation().get(), 1);
        assert_eq!(
            fixture
                .home
                .world()
                .inspection_port()
                .recovery_snapshot()
                .unwrap()
                .installed(),
            0
        );
        let contracts = original::contracts(&fixture, &execution, resumed.clone());
        let native = fixture.home.relational().runtime();
        let authority = fixture
            .home
            .relational()
            .world_recovery_authority()
            .unwrap();
        let grant = fixture
            .home
            .admit_home_program_transition_catalog(&execution)
            .unwrap();
        request_controls::refuse(native, authority, &grant, &contracts);

        let wrong = native
            .prepare_home_program_contracts(
                &execution,
                source.basis().relational_basis(),
                native.config().schema.registry.clone(),
                Vec::new(),
                Vec::new(),
            )
            .unwrap();
        let refused = authority
            .readmit_home_program_catalogs(&execution, native, &grant, &wrong)
            .unwrap_err();
        assert!(matches!(refused.reason(), Denial::TargetCatalogMismatch));
        assert!(!refused.retains_acquired_custody());
        drop(refused);
        let restored = authority
            .readmit_home_program_catalogs(&readmission_execution, native, &grant, &contracts)
            .unwrap();
        assert!(readmission_memory.charged_memory_bytes() > 0);
        assert_eq!(restored.durable_native_steps(), 1);
        assert_eq!(restored.current(), source.basis().relational_basis());
        assert_ne!(restored.predecessor(), restored.current());
        let repeated = authority
            .readmit_home_program_catalogs(&execution, native, &grant, &contracts)
            .unwrap();
        assert_eq!(repeated.current(), restored.current());
        let competing = original::contracts(&fixture, &execution, resumed.clone());
        assert_eq!(competing.catalog_digest(), contracts.catalog_digest());
        assert!(!competing
            .catalog_receipt()
            .same_inventory(&contracts.catalog_receipt()));
        let refused = authority
            .readmit_home_program_catalogs(&execution, native, &grant, &competing)
            .unwrap_err();
        assert!(matches!(refused.reason(), Denial::ConflictingInventory));
        assert!(refused.retains_acquired_custody());
        drop((refused, repeated, competing, wrong));
        assert_eq!(resumed.load(Ordering::SeqCst), 0);
        assert_eq!(
            fixture
                .home
                .world()
                .observation_port()
                .observe_root_product_branch()
                .unwrap(),
            source
        );

        let mut assigned = fixture
            .home
            .admit_unopened_home_program_transition_step(
                1,
                &source,
                HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
                2,
                HomeProgramTransitionTargetMeaning::from_recorded(0, contracts.catalog_digest()),
                &execution,
            )
            .unwrap();
        let cancellation = RuntimeWorldCancellationSource::new();
        let permit = assigned.take_execution().unwrap();
        let pending = fixture
            .home
            .world()
            .publication_port()
            .prepare_readmitted_home_transition_step(
                &execution,
                source.clone(),
                RelationalTransactionIntent::ordinary(),
                assigned.plan(),
                1,
                assigned.original(),
                assigned.admission(),
                permit,
                &cancellation.token(),
                None,
            )
            .unwrap();
        let mut opened = pending.advance(&execution);
        let opened = loop {
            match opened {
                WorldOpenAttemptOutcome::Opened(opened) => break opened,
                WorldOpenAttemptOutcome::Pending(pending) => opened = pending.advance(&execution),
                other => panic!("the actual missing original suffix must open: {other:?}"),
            }
        };
        let association =
            RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
        let mut transaction = native
            .begin_home_program_step(
                &execution,
                source.basis().relational_basis(),
                RelationalHomeProgramStepContracts::AfterTarget,
                assigned.plan(),
                HomeProgramTransitionStepAssociation::from_recorded(assigned.plan().identity(), 1),
                assigned.original(),
                &association,
                assigned.admission(),
            )
            .unwrap();
        transaction
            .push_batch(crate::batches::create(1), execution.allocation_policy())
            .unwrap();
        let proposal = transaction.validate(&execution, native).unwrap();
        assert!(
            resumed.load(Ordering::SeqCst) > 0,
            "the missing suffix uses the restored actual target executors"
        );
        let prepared = opened.prepare_native(native, &execution, proposal).unwrap();
        let performed = fixture.publish(&execution, prepared, &cancellation.token());
        assert_eq!(performed.new_product_head().reference_generation().get(), 2);
        let mut consumed = performed.consume();
        let current = consumed.take_successor_observation().unwrap();
        drop(consumed);
        assert_eq!(current.reference_generation().get(), 2);
        let proof = fixture
            .home
            .relational()
            .runtime()
            .durability_authority()
            .discover_associated_effects(&execution, &original_receipt.0)
            .unwrap();
        assert_eq!(proof.len(), 1);
        assert_eq!(proof[0].receipt(), &original_receipt.1);
        drop((proof, current, source, restored, grant, assigned, contracts));
        close(fixture.home, &execution);
    }
    assert_eq!(readmission_memory.charged_memory_bytes(), 0);
    let final_counter = Arc::new(AtomicUsize::new(0));
    let fixture = original::reopen(directory.path(), &execution);
    let contracts = original::contracts(&fixture, &execution, final_counter.clone());
    let grant = fixture
        .home
        .admit_home_program_transition_catalog(&execution)
        .unwrap();
    let restored = fixture
        .home
        .relational()
        .world_recovery_authority()
        .unwrap()
        .readmit_home_program_catalogs(
            &execution,
            fixture.home.relational().runtime(),
            &grant,
            &contracts,
        )
        .unwrap();
    assert_eq!(restored.durable_native_steps(), 2);
    assert_eq!(final_counter.load(Ordering::SeqCst), 0);
    assert_eq!(
        fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap()
            .reference_generation()
            .get(),
        2
    );
    drop((restored, grant, contracts));
    close(fixture.home, &execution);
}
