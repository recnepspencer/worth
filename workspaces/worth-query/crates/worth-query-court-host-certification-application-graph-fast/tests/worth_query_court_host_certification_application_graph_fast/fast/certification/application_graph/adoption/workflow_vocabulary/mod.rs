pub(crate) use worth_query_worlds::facade::certification::application_graph::adoption::workflow_vocabulary::*;

pub(crate) mod support;

#[test]
fn an_adopted_branch_runs_workflows_through_its_new_program_vocabulary() {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<RetentionProgramP1>(&mut application);
    let main = application.current_world();
    let initial = application.workflow_spec().program_revision();
    assert_eq!(
        application
            .with_host_advancement(|host_phase| application
                .vocabulary()
                .workflow_spec_on(&host_phase, main)
                .expect("workflow branch remains admitted")
                .program_revision())
            .expect("installed host call opens"),
        initial,
        "a supported vocabulary serves no branch until the branch adopts its program",
    );
    adopt_second_program(&application, main);

    let second = application
        .with_host_advancement(|host_phase| {
            application
                .vocabulary()
                .workflow_spec_on(&host_phase, main)
                .expect("workflow branch remains admitted")
        })
        .expect("installed host call opens");
    assert_eq!(second.program_revision(), second_revision(&application));
    assert_ne!(second.program_revision(), initial);

    let definition = expect_published(publish(&application, main, "completed", 84_000));
    let instance = match start_instance(&application, definition, 84_001)
        .expect("the P1 instance prepares on the adopted branch")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the P1 instance did not start: {other:?}"),
    };
    assert_eq!(instance.program_revision(), second.program_revision());
    match advance_instance(&application, instance, 84_002)
        .expect("the P1 advance prepares on the adopted branch")
    {
        WorkflowProgressOutcome::Completed(transition) => {
            assert_eq!(transition.node_path(), "completed");
            assert!(transition.terminal());
        }
        other => panic!("the P1 instance did not complete: {other:?}"),
    }

    // A definition bound to the program the branch left cannot publish there.
    let absent = WorkflowDefinitionExpectedPredecessor::Absent;
    let initial_spec = application.workflow_spec();
    let stale = publish_bound(&application, initial_spec, main, "stale", 84_003, absent);
    expect_stale_definition_vocabulary(stale);
}

#[test]
fn a_sibling_keeps_its_program_vocabulary_after_its_parent_adopts() {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<RetentionProgramP1>(&mut application);
    let main = application.current_world();
    let sibling = fork_sibling(&application, main);
    adopt_second_program(&application, main);

    let vocabulary = application.vocabulary();
    assert_eq!(
        application
            .with_host_advancement(|host_phase| vocabulary
                .workflow_spec_on(&host_phase, sibling)
                .expect("workflow branch remains admitted")
                .program_revision())
            .expect("installed host call opens"),
        application.workflow_spec().program_revision(),
    );
    let absent = WorkflowDefinitionExpectedPredecessor::Absent;
    let parent_spec = application
        .with_host_advancement(|host_phase| {
            vocabulary
                .workflow_spec_on(&host_phase, main)
                .expect("workflow branch remains admitted")
        })
        .expect("installed host call opens");
    let early = publish_bound(&application, parent_spec, sibling, "early", 84_100, absent);
    expect_stale_definition_vocabulary(early);
    expect_published(publish(&application, sibling, "unchanged", 84_101));
}

#[test]
fn an_adopted_branch_without_its_program_vocabulary_refuses_workflow_work() {
    let mut application = publish_workflow_on_first_program();
    let main = application.current_world();
    adopt_second_program(&application, main);
    assert_eq!(
        application
            .with_host_advancement(|host_phase| application
                .vocabulary()
                .workflow_spec_on(&host_phase, main)
                .expect("workflow branch remains admitted")
                .program_revision())
            .expect("installed host call opens"),
        application.workflow_spec().program_revision(),
        "a branch whose program has no installed vocabulary falls back to the initial one",
    );
    expect_stale_definition_vocabulary(publish(&application, main, "unserved", 84_150));

    // The refusal claimed nothing: once P1's vocabulary is supported, the
    // same key publishes through it.
    support_workflow_program::<RetentionProgramP1>(&mut application);
    expect_published(publish(&application, main, "unserved", 84_150));
}

#[test]
fn sibling_workflow_lineages_stay_on_their_own_branch() {
    let application = publish_workflow_on_first_program();
    let main = application.current_world();
    let sibling = fork_sibling(&application, main);

    let on_sibling = expect_published(publish(&application, sibling, "completed", 84_200));
    let on_main = expect_published(publish(&application, main, "completed", 84_201));
    // Each branch's current definition is its own publication.
    let replace = |branch, terminal, idempotency, own| {
        let predecessor = WorkflowDefinitionExpectedPredecessor::Published(own);
        let spec = application
            .with_host_advancement(|host_phase| {
                application
                    .vocabulary()
                    .workflow_spec_on(&host_phase, branch)
                    .expect("workflow branch remains admitted")
            })
            .expect("installed host call opens");
        publish_bound(
            &application,
            spec,
            branch,
            terminal,
            idempotency,
            predecessor,
        )
    };
    expect_published(replace(sibling, "settled", 84_202, on_sibling));
    expect_published(replace(main, "settled", 84_203, on_main));
}
