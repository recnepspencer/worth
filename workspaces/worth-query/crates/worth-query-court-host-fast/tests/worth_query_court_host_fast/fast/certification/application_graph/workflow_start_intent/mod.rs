pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_start_intent::*;

#[test]
fn original_start_locator_precedes_effects_and_matches_the_original_commit() {
    let application = publish_workflow_on_first_program();
    let definition = definition(
        &application,
        "original",
        WorkflowDefinitionExpectedPredecessor::Absent,
        6100,
    );
    let (original, performed) = {
        let runtime = application.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(runtime.installed_schema(), &scope);
        let request = runtime.request(&principal, &scope);
        let before = application
            .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
            .unwrap();
        let prepared = prepare(&application, definition.clone(), 6101);
        let original: Locator = prepared.discovery_intent();
        let equal = prepare(&application, definition.clone(), 6101);
        assert_eq!(original, equal.discovery_intent());
        drop(equal);
        let abandoned = prepare(&application, definition.clone(), 6102);
        let intended_only = abandoned.discovery_intent();
        drop(abandoned);
        assert_ne!(original, intended_only);
        let after = application
            .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
            .unwrap();
        assert_eq!(
            before.selected_commit(),
            after.selected_commit(),
            "preparing, cloning and abandoning intent publish no instance"
        );
        let performed = started(prepared.execute());
        assert!(!performed.replayed());
        assert!(original
            .matches_performed_start_for_test(&performed)
            .unwrap());
        assert!(!intended_only
            .matches_performed_start_for_test(&performed)
            .unwrap());
        (original, performed)
    }; // Principal, request, scopes and retained observations have ended.
    let instance = performed.instance().clone();
    drop(instance);
    let replay = started(prepare(&application, definition, 6101).execute());
    assert!(replay.replayed());
    assert_eq!(
        performed.receipt().commit_reference(),
        replay.receipt().commit_reference()
    );
    assert!(original.matches_performed_start_for_test(&replay).unwrap());
    drop(performed);
    drop(replay);
    assert_eq!(original, original.clone());
}

#[test]
fn changed_definition_and_successor_creation_do_not_replace_the_source_intent() {
    let application = publish_workflow_on_first_program();
    let first = definition(
        &application,
        "original",
        WorkflowDefinitionExpectedPredecessor::Absent,
        6120,
    );
    let prepared = prepare(&application, first.clone(), 6121);
    let source_intent = prepared.discovery_intent();
    let source = started(prepared.execute());
    let target = definition(
        &application,
        "resumed",
        WorkflowDefinitionExpectedPredecessor::Published(first),
        6122,
    );
    let changed = prepare(&application, target.clone(), 6121);
    assert_ne!(source_intent, changed.discovery_intent());
    drop(changed);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let migration = runtime
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent { input: input() })
        .without_source()
        .idempotency(&6123_u64)
        .prepare_workflow_instance_migration(
            &application,
            source.instance().clone(),
            target,
            "resumed",
        )
        .expect("the real native source migrates to the target terminal");
    let successor_intent = migration.discovery_intent();
    assert_ne!(source_intent, successor_intent);
    let successor = started(migration.execute());
    assert!(source_intent
        .matches_performed_start_for_test(&source)
        .unwrap());
    assert!(successor_intent
        .matches_performed_start_for_test(&successor)
        .unwrap());
    assert!(!source_intent
        .matches_performed_start_for_test(&successor)
        .unwrap());
}

#[test]
fn native_fork_continuations_keep_the_original_source_and_distinguish_forks() {
    let application = publish_workflow_on_first_program();
    let definition = definition(
        &application,
        "original",
        WorkflowDefinitionExpectedPredecessor::Absent,
        6140,
    );
    let prepared = prepare(&application, definition.clone(), 6141);
    let source_intent = prepared.discovery_intent();
    let source = started(prepared.execute());
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let mut successors = Vec::new();
    for _ in 0..2 {
        let fork = application
            .with_host_advancement(|phase| {
                runtime
                    .branches()
                    .fork(runtime.current_world())
                    .components(|components| {
                        components.fork_relational().reuse_exact_signal_basis()
                    })
                    .create(&phase)
                    .expect("native fork is issued")
            })
            .unwrap();
        let continuation = runtime
            .request(&principal, &scope)
            .on_branch(fork)
            .mutate(WorkflowInstanceStartIntent { input: input() })
            .without_source()
            .idempotency(&6142_u64)
            .prepare_workflow_fork_continuation(
                &application,
                source.instance().clone(),
                definition.clone(),
                "original",
            )
            .expect("the copied native source prepares its fork successor");
        let intent = continuation.discovery_intent();
        assert_ne!(source_intent, intent);
        let performed = started(continuation.execute());
        assert_eq!(performed.instance().branch(), fork);
        assert!(intent.matches_performed_start_for_test(&performed).unwrap());
        assert!(!source_intent
            .matches_performed_start_for_test(&performed)
            .unwrap());
        successors.push(intent);
    }
    assert_ne!(successors[0], successors[1]);
    assert!(source_intent
        .matches_performed_start_for_test(&source)
        .unwrap());
}
