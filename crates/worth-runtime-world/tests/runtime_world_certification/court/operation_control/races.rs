use super::*;
use std::num::NonZeroUsize;
#[test]
fn mixed_same_head_race_has_one_winner_and_preserves_signal_loser() {
    let mut court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let signal_attempt = signal_prepared(&court, &root, &token);
    let relational_attempt = court.prepare_cargo(&root, "5");
    let mut context = court.context(&root);
    let port = court.world.publication_port();
    let control = court.world.operation_control();
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let signal_before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .canonical_movements();
    std::thread::scope(|scope| {
        let pause = control.pause_before_product_compare(NonZeroUsize::new(1).unwrap());
        let worker = scope.spawn(move || {
            port.execute_with_signal(signal_attempt, &mut context, &token, |_| Ok(()))
        });
        assert!(pause.wait_until_reached(WAIT));
        let inspection = court.world.inspection_port();
        let pin = RuntimeWorldRetentionKey::relational(root.basis());
        assert_eq!(
            inspection
                .inspect_retention(&pin)
                .unwrap()
                .unwrap()
                .dependencies()
                .get(ComponentBasisDependencyClass::ActivePublicationAttempt),
            1
        );
        assert_eq!(
            signal
                .owner_service_cost_snapshot()
                .unwrap()
                .canonical_movements(),
            signal_before + 1,
            "direct Signal movement before product CAS"
        );
        assert_eq!(
            court.observe(&root).selected_commit(),
            root.selected_commit(),
            "reader before product movement"
        );
        let (read, commands) = mpsc::channel();
        let (samples, observed) = mpsc::channel();
        let observations = court.world.observation_port();
        let branch = root.branch_identity().clone();
        let reader = scope.spawn(move || {
            for _ in 0..3 {
                commands.recv_timeout(WAIT).expect("bounded reader command");
                samples
                    .send(observations.observe_product_branch(&branch).unwrap())
                    .unwrap();
            }
        });
        read.send(()).unwrap();
        let old = observed.recv_timeout(WAIT).unwrap();
        assert_eq!(old.selected_commit(), root.selected_commit());
        read.send(()).unwrap();
        let winner = court.world.publication_port().execute_without_signal(
            relational_attempt,
            &RuntimeWorldCancellationSource::new().token(),
        );
        let RuntimeWorldPublicationOutcome::Performed(winner) = winner else {
            panic!("Relational contender wins product head")
        };
        let head = court.observe(&root);
        assert_eq!(head.selected_commit(), winner.commit().identity());
        assert_eq!(
            head.basis().signal_basis().descriptor(),
            root.basis().signal_basis().descriptor(),
            "new product truth retains old exact Signal basis"
        );
        let direct = court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .1;
        assert_eq!(
            direct.admission_identity(),
            head.basis().relational_basis().admission_identity()
        );
        read.send(()).unwrap();
        let overlapping = observed.recv_timeout(WAIT).unwrap();
        let new = observed.recv_timeout(WAIT).unwrap();
        assert_eq!(new.selected_commit(), head.selected_commit());
        let expected = if overlapping.selected_commit() == root.selected_commit() {
            &root
        } else {
            assert_eq!(overlapping.selected_commit(), head.selected_commit());
            &head
        };
        assert_eq!(
            overlapping.basis().relational_basis().admission_identity(),
            expected.basis().relational_basis().admission_identity()
        );
        assert_eq!(
            overlapping.basis().signal_basis().admission_identity(),
            expected.basis().signal_basis().admission_identity()
        );
        reader.join().unwrap();
        pause.release();
        let RuntimeWorldPublicationOutcome::ProductUnpublished(loser) = worker.join().unwrap()
        else {
            panic!("Signal moved but product head changed")
        };
        assert_eq!(loser.owner_effect_count(), 1);
        let dependencies = inspection
            .inspect_retention(&pin)
            .unwrap()
            .unwrap()
            .dependencies();
        assert_eq!(
            dependencies.get(ComponentBasisDependencyClass::ActivePublicationAttempt),
            0
        );
        assert_eq!(
            dependencies.get(ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects),
            1
        );
        let costs = loser
            .product_comparison_costs()
            .expect("actual final comparison");
        assert_eq!((costs.cas_attempts(), costs.cas_losses()), (1, 1));
        assert_eq!(
            loser.cause(),
            ProductUnpublishedCause::ProductPublicationLost
        );
        let reference = signal
            .issue_managed_branch_reference(root.basis().signal_basis())
            .unwrap();
        let direct_signal = signal.observe_current(&reference).unwrap();
        assert_ne!(
            direct_signal.descriptor(),
            head.basis().signal_basis().descriptor()
        );
        assert_eq!(
            court.observe(&root).selected_commit(),
            head.selected_commit()
        );
        clean(
            &court,
            RuntimeWorldPublicationOutcome::ProductUnpublished(loser),
        );
        drop(winner.consume());
    });
    drop(root);
    court.finish();
}
