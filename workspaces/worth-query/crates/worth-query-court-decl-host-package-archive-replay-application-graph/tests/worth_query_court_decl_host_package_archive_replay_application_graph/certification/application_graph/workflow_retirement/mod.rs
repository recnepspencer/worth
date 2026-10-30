pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_retirement::*;

#[test]
fn retirement_stops_new_starts_while_pinned_work_completes_and_the_lineage_reopens() {
    let application = publish_workflow_on_first_program();
    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            terminal_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            401,
        ),
    );
    let waiting = expect_started(start_instance(&application, first.clone(), 402));
    let second = expect_published(
        "second revision",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(first.clone()),
            403,
        ),
    );

    let retired = expect_retired(retire_definition(&application, second.clone(), 404));
    assert!(!retired.replayed());
    assert_eq!(retired.definition(), &second);
    expect_retired_start(start_instance(&application, second.clone(), 405));
    expect_retired_start(start_instance(&application, first.clone(), 406));

    let replay = expect_retired(retire_definition(&application, second.clone(), 404));
    assert!(
        replay.replayed(),
        "an exact retirement retry recovers its receipt"
    );
    assert_eq!(
        replay.receipt().product_branch(),
        retired.receipt().product_branch()
    );
    expect_stale_retirement(retire_definition(&application, second.clone(), 407));
    expect_stale_retirement(retire_definition(&application, first.clone(), 408));

    match advance_instance(&application, waiting, 409)
        .expect("pinned work prepares against its retained definition")
    {
        WorkflowProgressOutcome::Completed(transition) => {
            assert_eq!(transition.node_path(), "completed");
            assert!(transition.terminal());
        }
        other => panic!("retirement must not strand pinned work, got {other:?}"),
    }

    expect_stale_publication(publish_definition(
        &application,
        terminal_definition("superseded"),
        WorkflowDefinitionExpectedPredecessor::Published(second.clone()),
        410,
    ));
    let reopened = expect_published(
        "reopened lineage",
        publish_definition(
            &application,
            terminal_definition("reopened"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            411,
        ),
    );
    assert_ne!(reopened.entity_id(), first.entity_id());
    assert_ne!(reopened.entity_id(), second.entity_id());
    let started = expect_started(start_instance(&application, reopened.clone(), 412));
    assert_eq!(started.start_node_path(), "reopened");
    let current = expect_superseded_start(start_instance(&application, second.clone(), 413));
    assert_eq!(
        current, reopened,
        "a refused start names the reopened definition"
    );

    let replay_after_reopen = expect_retired(retire_definition(&application, second, 404));
    assert!(
        replay_after_reopen.replayed(),
        "reopening the lineage must not hide an exact retirement replay"
    );
    expect_stale_publication(publish_definition(
        &application,
        terminal_definition("second-reopen"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        414,
    ));
}

pub(crate) mod contention;

pub(crate) mod denial;

pub(crate) mod discovery;

pub(crate) mod fork_definitions;
