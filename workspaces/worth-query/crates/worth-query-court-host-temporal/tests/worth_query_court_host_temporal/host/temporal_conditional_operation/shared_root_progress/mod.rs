pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::shared_root_progress::*;

#[test]
fn shared_host_root_reads_while_real_bridge_conditional_work_is_parked() {
    let host_request =
        worth_query_testkit_host::facade::host::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let contacts = ContactCounters::default();
    let (predicate, panic) = Predicate::controlled(contacts.clone());
    let (entered, entry) = mpsc::channel();
    let (resume, resumption) = mpsc::channel();
    let world = CourtroomWorld::publish_with_predicate(
        "ready",
        0,
        contacts,
        ParkedPredicate {
            predicate,
            entered,
            resume: Mutex::new(Some(resumption)),
        },
        panic,
        None,
        None,
        None,
        1,
        true,
    );
    let application = &world.application;
    let clock = &world.clock;
    std::thread::scope(|scope| {
        let conditional = scope.spawn(|| {
            let outcome = application
                .with_host_advancement(|phase| {
                    application
                        .on_branch(application.current_world())
                        .select(&phase)
                        .unwrap()
                        .conditional_clock(
                            &phase
                                .checked_execution_request_for(application.product_runtime())
                                .expect("phase belongs to application"),
                            clock,
                        )
                        .unwrap()
                })
                .expect("installed host call opens")
                .observe(request_execution);
            let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
                outcome
            else {
                panic!("the real conditional operation must complete after release")
            };
            assert_eq!(receipt.committed_operation_count(), 1);
            assert_eq!(
                receipt.execution_provenance()[0].signal_decision(),
                Some(primary_graph::WorthQueryConditionalSignalDecision::Eligible)
            );
        });
        let entered = entry.recv_timeout(Duration::from_secs(5));
        if entered.is_err() {
            let _ = resume.send(());
            conditional
                .join()
                .unwrap_or_else(|worker_panic| std::panic::resume_unwind(worker_panic));
            panic!("conditional work never reached the real Bridge predicate");
        }
        let (read, completed) = mpsc::channel();
        let reader = scope.spawn(move || {
            let product = application
                .with_host_advancement(|phase| {
                    application
                        .on_branch(application.current_world())
                        .select(&phase)
                        .is_ok()
                })
                .expect("installed host call opens");
            let entity = application
                .with_host_advancement(|phase| {
                    application
                        .on_branch(application.current_world())
                        .select(&phase)
                        .expect("the selected product branch remains admitted")
                        .resolve_entity(
                            &phase,
                            IntentIdentityField::reference(),
                            "intent-1".to_string(),
                            &request_scope(),
                            primary_graph::WorthQueryPrincipalResolutionMode::Certification,
                        )
                })
                .expect("installed host call opens");
            read.send((product, entity.is_ok())).unwrap();
        });
        let progress = completed.recv_timeout(Duration::from_secs(5));
        // Release before assertions so a broken concurrency boundary fails
        // without leaving a scoped worker blocked during unwinding.
        let _ = resume.send(());
        conditional
            .join()
            .unwrap_or_else(|worker_panic| std::panic::resume_unwind(worker_panic));
        reader
            .join()
            .unwrap_or_else(|worker_panic| std::panic::resume_unwind(worker_panic));
        assert_eq!(progress.unwrap(), (true, true));
    });
}
