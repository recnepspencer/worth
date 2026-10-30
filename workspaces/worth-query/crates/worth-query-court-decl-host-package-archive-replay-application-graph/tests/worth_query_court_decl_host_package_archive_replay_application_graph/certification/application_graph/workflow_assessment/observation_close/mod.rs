pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_assessment::observation_close::*;

#[test]
fn closing_one_observer_leaves_the_other_settling_the_shared_run() {
    let (application, instance) = awaiting_assessment(94_000);
    let mut first = observe(&application, &instance, 94_010, 1);
    in_flight(&application, &instance, &mut first);
    let mut second = observe(&application, &instance, 94_011, 1);
    let joined = second
        .notifications()
        .expect("an open observation has notifications")
        .generation();
    assert!(
        joined > 0,
        "the second observer joins the run already in flight"
    );
    first.close();
    closed(first.notifications());
    closed(progress(&application, &instance, &mut first));

    let settled = settle(&application, &instance, &mut second)
        .expect("the remaining observer settles the run the closed one started");
    accepted_past_assessment(&application, &instance, &settled, 94_012);
}

#[test]
fn closing_every_observer_leaves_the_instance_awaiting_its_assessment() {
    let (application, instance) = awaiting_assessment(94_100);
    let mut closed_observer = observe(&application, &instance, 94_110, 1);
    in_flight(&application, &instance, &mut closed_observer);
    closed_observer.close();
    drop(observe(&application, &instance, 94_111, 64));
    drop(closed_observer);
    assert_awaiting(&application, &instance, 94_112);

    let mut later = observe(&application, &instance, 94_113, 64);
    let settled = settle(&application, &instance, &mut later)
        .expect("a later observer settles the released assessment");
    accepted_past_assessment(&application, &instance, &settled, 94_114);
    drop(later);
    match cancel_instance(&application, instance, 94_115).expect("the cancellation prepares") {
        WorkflowInstanceCancellationOutcome::Cancelled(ended) => {
            assert!(
                !ended.replayed(),
                "closing observers did not end the instance"
            )
        }
        other => panic!("the instance must still cancel: {other:?}"),
    }
}

#[test]
fn an_observer_holds_at_most_one_pending_wake_for_its_run() {
    let (application, instance) = awaiting_assessment(94_200);
    let watcher = observe(&application, &instance, 94_210, 64);
    let mut settler = observe(&application, &instance, 94_211, 64);
    let notifications = watcher
        .notifications()
        .expect("an open observation has notifications");
    let before = notifications.generation();

    settle(&application, &instance, &mut settler).expect("the shared run settles");
    let after = notifications.generation();
    assert!(
        after > before + 1,
        "the shared run changed state more than once: {before} -> {after}",
    );
    assert_eq!(
        notifications.wait_after(before),
        after,
        "every change since the observer last looked is one pending wake",
    );
}
