use std::sync::mpsc;

use crate::mvcc::{PreparedRelationalCommitCandidate, RelationalPublicationOutcome};
use crate::runtime::{
    RelationalRuntime, RelationalRuntimeAdmissionHoldDenial as Denial,
    RelationalRuntimeAdmissionPosture,
};
use crate::tests::support::{
    batch_create, create_entity, runtime_with_test_schema, test_owner_begin_transaction_for_main,
};

fn candidate(runtime: &RelationalRuntime) -> PreparedRelationalCommitCandidate {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch_create("hold-candidate-write"))
        .unwrap();
    runtime.prepare_branch_transaction(transaction).unwrap()
}

#[test]
fn outstanding_candidate_refuses_hold_and_can_still_publish() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "candidate-hold-anchor");
    let prepared = candidate(&runtime);
    assert_eq!(
        runtime.try_hold_admission().into_result().unwrap_err(),
        Denial::PreparedCandidatesOutstanding
    );
    assert_eq!(
        runtime.owner_binding().admission_posture(),
        RelationalRuntimeAdmissionPosture::Open
    );
    let performed = match runtime.publication_port().compare_and_publish(prepared) {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("refused hold must preserve publication: {outcome:?}"),
    };
    let committed = runtime
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let abandoned = candidate(&runtime);
    assert_eq!(
        runtime.try_hold_admission().into_result().unwrap_err(),
        Denial::PreparedCandidatesOutstanding
    );
    assert_eq!(
        runtime.owner_binding().admission_posture(),
        RelationalRuntimeAdmissionPosture::Open
    );
    drop(abandoned);
    runtime.try_hold_admission().unwrap().seal();
}

#[test]
fn deferred_settlement_refuses_hold_preserves_custody_and_settles_afterward() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "deferred-hold-anchor");
    let prepared = candidate(&runtime);
    let performed = match runtime.publication_port().compare_and_publish(prepared) {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("candidate must perform: {outcome:?}"),
    };
    runtime.fail_next_durable_append_for_test();
    let error = runtime
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap_err();
    let settlement = error
        .deferred_settlement()
        .expect("append failure retains the performed route");
    let commit_id = settlement.commit().commit_id;
    let binding = runtime.publication_binding();
    let owner_loss = binding.pending_settlement_owner_loss_count();
    assert_eq!(
        runtime.try_hold_admission().into_result().unwrap_err(),
        Denial::PerformedPublicationRequiresSettlement(commit_id)
    );
    assert_eq!(
        runtime.owner_binding().admission_posture(),
        RelationalRuntimeAdmissionPosture::Open
    );
    assert_eq!(binding.pending_settlement_count(), 1);
    assert!(runtime
        .settlement_port()
        .retains_pending_settlement(commit_id));
    assert_eq!(binding.pending_settlement_owner_loss_count(), owner_loss);
    assert_eq!(
        runtime
            .repair_deferred_publication_settlement(settlement)
            .unwrap()
            .commit_id,
        commit_id
    );
    assert!(!runtime
        .settlement_port()
        .retains_pending_settlement(commit_id));
    runtime.try_hold_admission().unwrap().seal();
    drop(runtime);
    assert_eq!(binding.pending_settlement_owner_loss_count(), owner_loss);
}

#[test]
fn admissions_waiting_on_a_refused_quiescence_check_proceed() {
    for publication in [false, true] {
        let runtime = runtime_with_test_schema();
        create_entity(&runtime, "refused-quiescence-anchor");
        let prepared = candidate(&runtime);
        let (prepared, performed) = if publication {
            match runtime.publication_port().compare_and_publish(prepared) {
                RelationalPublicationOutcome::Performed(performed) => (None, Some(performed)),
                outcome => panic!("candidate must perform: {outcome:?}"),
            }
        } else {
            (Some(prepared), None)
        };
        let identity = runtime.main_branch_identity();
        let basis = runtime.owner_component_services().basis_port();
        let binding = runtime.owner_binding();
        let (held, observed_hold) = mpsc::channel();
        let (resume, continue_hold) = mpsc::channel();
        let (waiting, observed_wait) = mpsc::channel();
        binding.install_test_hold_start_pause(held, continue_hold);
        binding.install_test_hold_wait_ack(waiting.clone());
        let holder = std::thread::spawn(move || {
            let mut runtime = runtime;
            let denial = runtime.try_hold_admission().into_result().unwrap_err();
            (runtime, denial)
        });
        observed_hold.recv().unwrap();
        let admission = std::thread::spawn(move || {
            let result = basis.observe_branch(&identity);
            waiting.send(false).unwrap();
            result
        });
        let first_waited = observed_wait.recv().unwrap();
        resume.send(()).unwrap();
        let (mut runtime, denial) = holder.join().unwrap();
        assert!(
            first_waited,
            "completion before waiting violates a temporary hold"
        );
        assert_eq!(
            binding.admission_posture(),
            RelationalRuntimeAdmissionPosture::Open
        );
        admission
            .join()
            .unwrap()
            .expect("refusal releases, never denies, the waiter");
        if let Some(performed) = performed {
            assert!(matches!(
                denial,
                Denial::PerformedPublicationRequiresSettlement(_)
            ));
            runtime.settle_performed_publication(performed).unwrap();
        } else {
            assert_eq!(denial, Denial::PreparedCandidatesOutstanding);
            drop(prepared);
        }
        runtime.try_hold_admission().unwrap().seal();
    }
}

#[test]
fn detached_candidate_cleanup_refuses_hold_until_reservations_are_released() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "cleanup-quiescence-anchor");
    let prepared = candidate(&runtime);
    let binding = runtime.publication_binding();
    let (cleaning, observed_cleanup) = mpsc::channel();
    let (resume, continue_cleanup) = mpsc::channel();
    binding.install_test_candidate_cleanup_pause(cleaning, continue_cleanup);
    std::thread::scope(|scope| {
        let cleanup = scope.spawn(move || drop(prepared));
        observed_cleanup.recv().unwrap();
        let outcome = runtime.try_hold_admission().into_result();
        resume.send(()).unwrap();
        assert_eq!(outcome.unwrap_err(), Denial::PreparedCandidatesOutstanding);
        cleanup.join().unwrap();
    });
    let hold = runtime.try_hold_admission().unwrap();
    let before = hold.native_checkpoint().unwrap();
    hold.seal();
    assert_eq!(
        before,
        runtime.durability_authority().native_checkpoint().unwrap()
    );
}

#[test]
fn busy_checkpoint_publication_gate_refuses_hold_without_changing_admission() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "busy-publication-gate");
    let routes = runtime.history.canonical_checkpoint_gate();
    let publication = routes.enter_fork().unwrap();
    assert_eq!(
        runtime.try_hold_admission().into_result().unwrap_err(),
        Denial::PublicationInFlight
    );
    assert_eq!(
        runtime.owner_binding().admission_posture(),
        RelationalRuntimeAdmissionPosture::Open
    );
    assert!(runtime.owner_binding().admit().is_some());
    drop(publication);
    runtime.try_hold_admission().unwrap().seal();
}
