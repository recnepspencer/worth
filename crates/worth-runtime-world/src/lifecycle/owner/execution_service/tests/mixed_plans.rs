use super::*;

/// Both exact owner effects complete against one product observation. The
/// product comparison, rather than either owner's latest head, selects truth.
#[test]
fn disjoint_owner_attempts_both_execute_and_only_one_product_head_moves() {
    for relational_wins in [true, false] {
        let (fixture, owner, expected) = setup();
        let relational = prepare_relational(&fixture, &owner, &expected, "mixed-plan-race");
        let signal = prepare_signal(&owner, &expected, None);
        let (relational, signal) = execute_while_signal_is_parked(&owner, relational, signal);
        // Direct owner observations establish that both components moved while
        // the independently observed product reference still selects the root.
        assert!(owner
            .state
            .relational
            .basis_port()
            .compare_current_exact(expected.basis().relational_basis())
            .is_err());
        assert!(owner
            .state
            .signal
            .basis_port()
            .compare_current_exact(expected.basis().signal_basis())
            .is_err());
        let cell = owner.state.branches.root_cell().unwrap();
        assert_eq!(cell.atomic_snapshot(), *expected.snapshot());
        let relational_basis = relational.successor_basis().unwrap().clone();
        let signal_basis = signal.successor_basis().unwrap().clone();
        owner
            .state
            .relational
            .basis_port()
            .compare_current_exact(relational_basis.relational_basis())
            .unwrap();
        owner
            .state
            .signal
            .basis_port()
            .compare_current_exact(signal_basis.signal_basis())
            .unwrap();
        assert_eq!(
            relational_basis.signal_basis().admission_identity(),
            expected.basis().signal_basis().admission_identity()
        );
        assert_eq!(
            signal_basis.relational_basis(),
            expected.basis().relational_basis()
        );
        let relational = relational.ready(relational_basis).unwrap();
        let signal = signal.ready(signal_basis).unwrap();
        let (winner, loser) = if relational_wins {
            (relational, signal)
        } else {
            (signal, relational)
        };
        let winner = match winner.publish(&cell, CompositeLateCancellationPosture::NotRequested) {
            crate::publication::RuntimeWorldPublicationOutcome::Performed(winner) => winner,
            other => panic!("first product CAS must perform: {other:?}"),
        };
        let counters = winner.cost_counters();
        assert_eq!(
            counters.relational_owner_contacts(),
            u64::from(relational_wins)
        );
        assert_eq!(
            counters.signal_owner_contacts(),
            u64::from(!relational_wins)
        );
        let loser = match loser.publish(&cell, CompositeLateCancellationPosture::NotRequested) {
            crate::publication::RuntimeWorldPublicationOutcome::ProductUnpublished(loser) => loser,
            other => panic!("losing owner effect must remain unpublished: {other:?}"),
        };
        assert_eq!(cell.atomic_snapshot(), *winner.new_product_head());
        assert_eq!(loser.owner_effect_count(), 1);
        assert_eq!(
            loser.cause(),
            ProductUnpublishedCause::ProductPublicationLost
        );
        assert_eq!(owner.recovery_record_count(), 1);
        let handle = loser.recovery_handle();
        drop(loser);
        assert!(owner.cleanup_recovery_handle(&handle).is_some());
        assert_eq!(owner.recovery_record_count(), 0);
        assert_eq!(owner.state.operation.active(), 0);
        drop(winner);
        drop(expected);
        let _report = owner.close().unwrap();
    }
}

/// Park inside the real Signal owner call. Relational must still finish its
/// independent owner leg before Signal is released; a cross-owner execution
/// lock would fail the bounded completion handshake.
fn execute_while_signal_is_parked(
    owner: &TestOwner,
    relational: PreparedCompositePublicationWithoutSignal,
    signal: PreparedCompositePublicationWithSignal,
) -> (OwnerExecutionSettlement, OwnerExecutionSettlement) {
    std::thread::scope(|scope| {
        let (reached, reached_rx) = sync_channel(1);
        let (resume, resume_rx) = sync_channel(1);
        let signal_worker = scope.spawn(move || {
            owner.execute_with_signal(
                signal,
                &mut (),
                &RuntimeWorldCancellationSource::new().token(),
                |_| {
                    reached.send(()).unwrap();
                    resume_rx
                        .recv_timeout(REHEARSAL_HANDSHAKE_BUDGET)
                        .expect("Signal owner is released after the sibling completion check");
                    Ok(())
                },
            )
        });
        let reached_result = reached_rx.recv_timeout(REHEARSAL_HANDSHAKE_BUDGET);
        if reached_result.is_err() {
            drop(resume);
            signal_worker.join().unwrap();
            panic!("real Signal owner did not reach its parked mutation");
        }
        let (completed, completed_rx) = sync_channel(1);
        let relational_worker = scope.spawn(move || {
            completed
                .send(execute_without_signal(owner, relational))
                .unwrap();
        });
        let relational_result = completed_rx.recv_timeout(REHEARSAL_HANDSHAKE_BUDGET);
        // Release and join both owners before asserting, even on timeout.
        let _ = resume.send(());
        relational_worker.join().unwrap();
        let signal_result = signal_worker.join().unwrap();
        (
            settled(relational_result.expect("Relational progresses while Signal is parked")),
            settled(signal_result),
        )
    })
}
