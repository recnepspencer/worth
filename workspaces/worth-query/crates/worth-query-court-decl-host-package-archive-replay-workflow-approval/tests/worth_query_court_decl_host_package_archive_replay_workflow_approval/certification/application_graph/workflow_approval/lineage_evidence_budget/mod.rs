pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_approval::lineage_evidence_budget::*;

#[test]
fn evidence_past_the_ceiling_is_refused_and_leaves_the_instance_waiting() {
    let application = bounded(1);
    let (_, instance) = proposed(&application, 95_000);
    evidence_refused(&application, &instance, 95_010);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    evidence_refused(&application, &instance, 95_020);
    assert_awaiting(&application, &instance, 95_030);
    let ended = cancelled(cancel_instance(&application, instance, 95_040));
    assert!(
        !ended.replayed(),
        "a refused assessment leaves the instance live"
    );
}

#[test]
fn a_lineage_retains_evidence_only_up_to_its_ceiling() {
    let application = bounded(EVIDENCE_BUDGET);
    let (_, instance) = proposed(&application, 95_100);
    let (retained, last) = spend_until_refused(&application, &instance, 95_103, 0);
    let (last, acceptance_key) = last.expect("the installed budget admits real evidence");
    let before_replay = application
        .runtime()
        .workflow_instance_progress_counters()
        .incremental_evidence_bytes();
    assert!(
        accept(&application, &instance, &last, acceptance_key),
        "a retry of the accepted assessment replays",
    );
    evidence_refused(&application, &instance, 95_170);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    assert!(
        accept(&application, &instance, &last, acceptance_key),
        "a cold retry of the accepted assessment replays",
    );
    assert_eq!(
        application
            .runtime()
            .workflow_instance_progress_counters()
            .incremental_evidence_bytes(),
        before_replay,
        "warm and cold replay retain no new evidence"
    );
    assert!(retained <= EVIDENCE_BUDGET);
    evidence_refused(&application, &instance, 95_180);
    assert_awaiting(&application, &instance, 95_190);
}

#[test]
fn a_fork_continuation_inherits_the_evidence_its_source_retained() {
    let application = bounded(EVIDENCE_BUDGET);
    let (definition, instance) = proposed(&application, 95_200);
    let (inherited, last) = spend_until_refused(&application, &instance, 95_203, 0);
    assert!(
        last.is_some(),
        "the source retains real evidence before its fork"
    );
    let fork = fork_of(&application, instance.branch());
    let successor = started(continue_on_fork(
        &application,
        fork,
        instance,
        definition,
        "propose",
        95_300,
    ))
    .instance()
    .clone();
    proposal::published_proposal(&application, successor.clone(), 95_301);
    spend_until_refused(&application, &successor, 95_310, inherited);
    evidence_refused(&application, &successor, 95_350);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    evidence_refused(&application, &successor, 95_360);
    assert_awaiting(&application, &successor, 95_370);
}
