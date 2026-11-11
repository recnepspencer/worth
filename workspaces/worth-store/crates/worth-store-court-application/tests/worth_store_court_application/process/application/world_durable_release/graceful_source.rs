//! Release preserves its original while a later incarnation edits the predecessor.
use super::*;

#[test]
fn released_native_effect_preserves_history_and_allows_a_fresh_edit_after_graceful_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let initial = bootstrap.product_branch().clone();
    drop(bootstrap);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        initial.clone(),
        crate::batches::create(0),
        &cancellation.token(),
    );
    let gate = fixture.home.physical_serving_for_test().certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterRootReplacement,
    );
    let publication = fixture.home.world().publication_port();
    let token = cancellation.token();
    let receipt = std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let request = crate::workload::serial_request();
            publication.execute_durable_without_signal(
                ExecutionRequest::serial(&request),
                prepared,
                &token,
            )
        });
        let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
        while !gate.await_arrival() {
            assert!(
                !worker.is_finished(),
                "publication must reach its actual Native seam"
            );
            assert!(
                std::time::Instant::now() < deadline,
                "Native publication seam timed out"
            );
        }
        cancellation.cancel();
        gate.release();
        let effects = match worker.join().unwrap() {
            WorldDurablePublicationOutcome::RecoveryRequired { custody, .. }
            | WorldDurablePublicationOutcome::ProductUnpublished(custody) => custody,
            other => panic!(
                "cancelled product movement retains the performed Native original: {other:?}"
            ),
        };
        let receipt = effects
            .component_results()
            .relational_settlement()
            .expect("actual Native publication precedes cancellation")
            .clone();
        drop(effects);
        receipt
    });
    let initial_commit = initial.selected_commit().ordinal();
    drop(initial);
    // Live unpublished business custody deliberately refuses graceful close.
    // Abandon the original owner; the durable Open and actual Native effect
    // remain for the following genuine receiver to release.
    drop(fixture);

    let fixture = open(directory.path(), &execution);
    let page = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let recovery = fixture.home.world().recovery_port();
    let claim = recovery
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
    assert_eq!(claim.native_receipt().as_ref(), Some(&receipt));
    assert_eq!(
        claim.expected_head().selected_commit().ordinal(),
        initial_commit
    );
    let mut outcome = recovery.release_recovered_effects(&execution, claim);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let released = loop {
        match outcome {
            WorldRecoveredReleaseOutcome::Released(released) => break released,
            WorldRecoveredReleaseOutcome::Pending(claim) => {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(1));
                outcome = recovery.release_recovered_effects(&execution, claim);
            }
            other => panic!("actual recovered original release refused: {other:?}"),
        }
    };
    assert_eq!(released.original().native_receipt(), Some(&receipt));
    drop((released, page));
    close(fixture.home, &execution);

    let fixture = open(directory.path(), &execution);
    let current = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(current.selected_commit().ordinal(), initial_commit);
    assert!(fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap()
        .rows()
        .is_empty());
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        current,
        crate::batches::create(1),
        &cancellation.token(),
    );
    let mut performed = fixture
        .publish(&execution, prepared, &cancellation.token())
        .consume();
    let successor = performed.take_successor_observation().unwrap();
    assert_eq!(successor.reference_generation().get(), 1);
    let selected = successor.selected_commit().ordinal();
    let later_receipt = performed
        .component_results()
        .relational_settlement()
        .unwrap()
        .clone();
    assert_ne!(later_receipt.commit_id, receipt.commit_id);
    assert_eq!(
        later_receipt.parents,
        Vec::new(),
        "the selected predecessor is the real Empty root"
    );
    drop((performed, successor));
    close(fixture.home, &execution);

    let fixture = open(directory.path(), &execution);
    let current = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(current.selected_commit().ordinal(), selected);
    assert_eq!(current.reference_generation().get(), 1);
    assert_eq!(
        fixture
            .home
            .relational()
            .runtime()
            .history()
            .immutable_commit_count(),
        2
    );
    let originals = fixture
        .home
        .world()
        .recovery_port()
        .released_original_page(&execution, None, maximum())
        .unwrap();
    assert_eq!(originals.rows().len(), 1);
    assert_eq!(originals.rows()[0].native_receipt(), Some(&receipt));
    drop((current, originals));
    close(fixture.home, &execution);
}
