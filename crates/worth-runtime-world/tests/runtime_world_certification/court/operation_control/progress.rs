use super::*;
use worth_signal::facade::branch::validate_signal_branch_name;
#[test]
fn unrelated_signal_branch_progresses_at_each_owner_park() {
    for boundary in [
        Boundary::OwnerLifecycleAdmission,
        Boundary::BranchRegistryLookup,
        Boundary::ExactBasisPreflight,
        Boundary::TargetCellAdmission,
        Boundary::BeforeCanonicalMovement,
        Boundary::AfterCanonicalMovement,
        Boundary::OutcomeConstruction,
    ] {
        let court = CompositeSupplyChainCourt::compile();
        let root = court.bootstrap();
        let token = RuntimeWorldCancellationSource::new().token();
        let intent = ProductBranchCreationIntent::from_source(
            "independent",
            ProductBranchCreationPlans::new(
                RelationalBranchCreationPlan::ReuseExact,
                SignalBranchCreationPlan::ForkExact {
                    target: validate_signal_branch_name("independent").unwrap(),
                },
            ),
        )
        .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(other) = court
            .world
            .branch_port()
            .create_product_branch(root.clone(), intent, &token)
            .unwrap()
        else {
            panic!("real independent branch")
        };
        let first = signal_prepared(&court, &root, &token);
        let second = signal_prepared(&court, &other, &token);
        let mut first_context = court.context(&root);
        let mut second_context = court.context(&other);
        let control = court.signal.owner_operation_control().unwrap();
        let port = court.world.publication_port();
        let second_port = port.clone();
        let first_token = token.clone();
        let (send, receive) = mpsc::sync_channel(1);
        std::thread::scope(|scope| {
            // This guard is inside the scope so assertion unwinding releases
            // the parked owner before scope's implicit worker joins.
            let pause = control.arm_pause_once(boundary);
            let first_worker = scope.spawn(move || {
                port.execute_with_signal(first, &mut first_context, &first_token, |_| Ok(()))
            });
            assert!(pause.wait_until_reached(WAIT), "owner park {boundary:?}");
            let second_worker = scope.spawn(move || {
                let result = second_port.execute_with_signal(
                    second,
                    &mut second_context,
                    &token,
                    |_| Ok(()),
                );
                send.send(result).unwrap();
            });
            let progressed = receive
                .recv_timeout(WAIT)
                .expect("unrelated product branch must progress while owner is parked");
            assert!(
                matches!(progressed, RuntimeWorldPublicationOutcome::Performed(_)),
                "unrelated branch at {boundary:?}"
            );
            clean(&court, progressed);
            pause.release();
            let resumed = first_worker.join().unwrap();
            assert!(
                matches!(resumed, RuntimeWorldPublicationOutcome::Performed(_)),
                "healthy parked operation performs after release at {boundary:?}"
            );
            clean(&court, resumed);
            second_worker.join().unwrap();
        });
        let report = court
            .world
            .branch_port()
            .retire_product_branch(&other)
            .unwrap();
        assert_eq!(report.owner_retirement_work().len(), 1);
        drop((root, other, report));
        court.finish();
    }
}
