use super::*;
use std::sync::mpsc;
use std::time::Duration;

/// Concurrent totals are kept separate from the returned per-attempt costs.
pub(super) fn run(width: usize) -> (u128, CompositePublicationCostCounters) {
    let mut court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let mut heads = vec![];
    let mut jobs = vec![];
    let token = RuntimeWorldCancellationSource::new().token();
    for index in 0..width {
        let head = population::child(&court, &root, &format!("writer-{index}"), true, true);
        let prepared = court
            .world
            .publication_port()
            .prepare_with_signal(
                head.clone(),
                CompositePublicationIntent::with_signal(None),
                &token,
                None,
            )
            .unwrap();
        jobs.push((prepared, court.context(&head)));
        heads.push(head);
    }
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .canonical_movements();
    let start = Instant::now();
    let costs = std::thread::scope(|scope| {
        let (ready, arrived) = mpsc::channel();
        let mut starts = vec![];
        let mut workers = vec![];
        for (prepared, mut context) in jobs {
            let port = court.world.publication_port();
            let token = token.clone();
            let (start, gate) = mpsc::channel();
            starts.push(start);
            let ready = ready.clone();
            workers.push(scope.spawn(move || {
                ready.send(()).unwrap();
                gate.recv_timeout(Duration::from_secs(5))
                    .expect("bounded writer start");
                let RuntimeWorldPublicationOutcome::Performed(done) =
                    port.execute_with_signal(prepared, &mut context, &token, |_| Ok(()))
                else {
                    panic!("independent writer must publish");
                };
                let costs = done.cost_counters();
                drop(done.consume());
                costs
            }));
        }
        for _ in 0..width {
            arrived
                .recv_timeout(Duration::from_secs(5))
                .expect("all writers offered before release");
        }
        for start in starts {
            start.send(()).unwrap();
        }
        workers
            .into_iter()
            .map(|w| w.join().unwrap())
            .collect::<Vec<_>>()
    });
    let elapsed = start.elapsed().as_nanos();
    for cost in &costs {
        assert_eq!(*cost, costs[0]);
        assert_eq!(
            (
                cost.relational_owner_contacts(),
                cost.signal_owner_contacts()
            ),
            (0, 1)
        );
        assert_eq!(
            (cost.cas_attempts(), cost.cas_wins(), cost.cas_losses()),
            (1, 1, 0)
        );
    }
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements()
            - before,
        width as u64
    );
    assert_eq!(
        court.observe(&root).selected_commit(),
        root.selected_commit(),
        "unrelated main product stays fixed"
    );
    for head in heads {
        let updated = court.observe(&head);
        assert_eq!(updated.reference_generation().get(), 1);
        let rel = court
            .records
            .runtime
            .observe_branch(head.basis().relational_basis().identity())
            .unwrap()
            .1;
        assert_eq!(
            rel.admission_identity(),
            head.basis().relational_basis().admission_identity()
        );
        let reference = signal
            .issue_managed_branch_reference(head.basis().signal_basis())
            .unwrap();
        let direct = signal.observe_current(&reference).unwrap();
        assert_eq!(
            updated.basis().signal_basis().admission_identity(),
            direct.admission_identity()
        );
        drop(
            court
                .world
                .branch_port()
                .retire_product_branch(&updated)
                .unwrap(),
        );
    }
    drop(root);
    court.finish();
    (elapsed, costs[0])
}
