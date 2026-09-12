use super::*;

#[test]
fn initial_schema_transition_preserves_retention_identity_exhaustion() {
    assert_eq!(
        branch_transition_denial(
            crate::branch::RelationalBranchCellDenial::RetentionIdentityExhausted,
        )
        .kind(),
        RelationalInitialSchemaInstallationDenialKind::RetentionIdentityExhausted,
    );
}
#[test]
fn retained_publication_port_rejects_candidate_from_before_empty_invariant_seal() {
    use crate::tests::support::{batch_create, runtime_with_test_schema, test_owner_main_basis};
    let mut runtime = runtime_with_test_schema();
    let port = runtime.publication_port();
    let basis = test_owner_main_basis(&runtime).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction.push_batch(batch_create("before-seal")).unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    assert_eq!(receipt.custom_invariant_generation(), 1);
    let sealed_basis = test_owner_main_basis(&runtime).unwrap();
    assert!(matches!(
        port.compare_and_publish(candidate),
        crate::mvcc::RelationalPublicationOutcome::Denied(
            crate::mvcc::RelationalPublicationDenial::StaleInvariantGeneration {
                expected_generation: 1,
                actual_generation: 0,
            }
        )
    ));
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        sealed_basis.descriptor()
    );
    assert!(runtime.history().latest_commit().is_none());
    let repeat = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap_err();
    assert_eq!(
        repeat.kind(),
        RelationalInitialSchemaInstallationDenialKind::InitialInvariantsAlreadySealed
    );
    let mut fresh = runtime
        .begin_branch_transaction(
            &sealed_basis,
            crate::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    fresh.push_batch(batch_create("after-seal")).unwrap();
    let fresh = runtime.prepare_branch_transaction(fresh).unwrap();
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        port.compare_and_publish(fresh)
    else {
        panic!("same retained publication port must accept a freshly validated generation");
    };
    runtime.settle_performed_publication(performed).unwrap();
    assert!(runtime.history().latest_commit().is_some());
    let denial = runtime.prepare_initial_schema_installation().unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalInitialSchemaInstallationDenialKind::RuntimeAlreadyCommitted
    );
}

#[test]
fn delayed_installation_rejects_published_unsettled_branch() {
    use crate::tests::support::{batch_create, runtime_with_test_schema, test_owner_main_basis};
    let mut runtime = runtime_with_test_schema();
    let basis = test_owner_main_basis(&runtime).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch_create("before-delayed-seal"))
        .unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    let publication = runtime.publication_port();
    let installation = runtime.prepare_initial_schema_installation().unwrap();
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        publication.compare_and_publish(candidate)
    else {
        panic!("candidate must publish before the delayed seal")
    };
    let denial = installation
        .install(RelationalSchemaRegistry::new())
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalInitialSchemaInstallationDenialKind::RuntimeAlreadyCommitted
    );
    assert_eq!(
        runtime
            .configuration
            .snapshot()
            .schema_contract_runtime
            .custom_invariant_generation,
        0
    );
    runtime.settle_performed_publication(performed).unwrap();
}

#[test]
fn retained_ports_cannot_cross_initial_root_and_configuration_cutover() {
    use crate::tests::support::{batch_create, runtime_with_test_schema, test_owner_main_basis};
    use std::sync::mpsc;
    use std::time::Duration;
    let mut runtime = runtime_with_test_schema();
    let basis = test_owner_main_basis(&runtime).unwrap();
    let mut prepared_transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    prepared_transaction
        .push_batch(batch_create("old-prepared"))
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(prepared_transaction)
        .unwrap();
    let mut pending_transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    pending_transaction
        .push_batch(batch_create("old-pending"))
        .unwrap();
    let preparation = runtime.preparation_port();
    let publication = runtime.publication_port();
    let (reached_tx, reached_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let mut installation = runtime.prepare_initial_schema_installation().unwrap();
    installation.transition_pause = Some((reached_tx, resume_rx));
    std::thread::scope(|threads| {
        let seal = threads.spawn(move || installation.install(RelationalSchemaRegistry::new()));
        reached_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();
        let preparation_started = started_tx.clone();
        let preparation_finished = finished_tx.clone();
        let prepare = threads.spawn(move || {
            preparation_started.send(()).unwrap();
            let result = preparation.prepare_branch_transaction(pending_transaction);
            preparation_finished.send(()).unwrap();
            result
        });
        let publish = threads.spawn(move || {
            started_tx.send(()).unwrap();
            let result = publication.compare_and_publish(candidate);
            finished_tx.send(()).unwrap();
            result
        });
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        // Release before asserting so even a failing regression cannot strand
        // installation or its scoped worker threads at the test barrier.
        let escaped = finished_rx.recv_timeout(Duration::from_millis(100)).is_ok();
        resume_tx.send(()).unwrap();
        assert_eq!(
            seal.join().unwrap().unwrap().custom_invariant_generation(),
            1
        );
        let prepared = prepare.join().unwrap();
        let published = publish.join().unwrap();
        assert!(
            !escaped,
            "retained ports must wait for the complete owner cutover"
        );
        assert!(
            prepared.is_err(),
            "old branch basis must not prepare after cutover"
        );
        assert!(matches!(
            published,
            crate::mvcc::RelationalPublicationOutcome::Denied(
                crate::mvcc::RelationalPublicationDenial::StaleInvariantGeneration {
                    expected_generation: 1,
                    actual_generation: 0,
                }
            )
        ));
    });
    assert!(runtime.history().latest_commit().is_none());
    let basis = test_owner_main_basis(&runtime).unwrap();
    let mut fresh = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    fresh
        .push_batch(batch_create("fresh-after-cutover"))
        .unwrap();
    let candidate = runtime
        .preparation_port()
        .prepare_branch_transaction(fresh)
        .unwrap();
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("fresh owner generation must publish after cutover")
    };
    runtime.settle_performed_publication(performed).unwrap();
}
