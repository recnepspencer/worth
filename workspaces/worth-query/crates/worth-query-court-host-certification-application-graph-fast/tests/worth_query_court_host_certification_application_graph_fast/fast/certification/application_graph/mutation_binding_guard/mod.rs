pub(crate) use worth_query_worlds::facade::certification::application_graph::mutation_binding_guard::*;

#[test]
fn a_program_built_by_one_binding_cannot_commit_under_another() {
    let application = publish_on_first_program();
    let runtime = application.runtime();
    let branch = application.current_world();
    let before = read_retention(runtime, branch);
    let (key, request) = (7_001_u64, input(before + 1));

    let program = candidate!(runtime, branch, ReviewedSetRetentionBinding, key, request);
    let outcome = application.compare_and_commit_program_action(
        program,
        &identities::<SetRetentionBinding>(&key, &request),
        std::convert::identity,
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("a program must not commit under a binding other than its handler's");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::MutationBindingMismatch
    );
    assert_eq!(
        read_retention(runtime, branch),
        before,
        "nothing was written"
    );

    let program = candidate!(runtime, branch, SetRetentionBinding, key, request);
    let outcome = application.compare_and_commit_program_action(
        program,
        &identities::<SetRetentionBinding>(&key, &request),
        std::convert::identity,
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the same request under its own binding commits: {outcome:?}"
    );
    assert_eq!(read_retention(runtime, branch), before + 1);
}

#[test]
fn one_key_and_input_do_not_replay_across_bindings() {
    let application = publish_on_first_program();
    let runtime = application.runtime();
    let branch = application.current_world();
    let (key, request) = (7_002_u64, input(read_retention(runtime, branch) + 1));

    let program = candidate!(runtime, branch, SetRetentionBinding, key, request);
    assert!(matches!(
        application.compare_and_commit_program_action(
            program,
            &identities::<SetRetentionBinding>(&key, &request),
            std::convert::identity,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    let (_, admission, _) = admit!(runtime, branch, SetRetentionBinding, key, request);
    let resolve = |binding: WorthQueryApplicationIdempotencyBinding| {
        application
            .with_host_advancement(|host_phase| {
                runtime
                    .resolve_admitted_application_idempotency(&host_phase, &admission, binding)
                    .expect("the key resolves")
                    .into_resolution()
            })
            .expect("installed host call opens")
    };
    assert!(
        matches!(
            resolve(idempotency::<SetRetentionBinding>(&key, &request)),
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(_)
        ),
        "the binding that committed replays its receipt"
    );
    let other = resolve(idempotency::<ReviewedSetRetentionBinding>(&key, &request));
    assert!(
        matches!(other, WorthQueryApplicationIdempotencyResolution::Unseen),
        "another binding's key namespace never replays this receipt: {other:?}"
    );
}

#[test]
fn a_program_decided_on_one_input_cannot_commit_under_another_inputs_idempotency() {
    let application = publish_on_first_program();
    let runtime = application.runtime();
    let branch = application.current_world();
    let before = read_retention(runtime, branch);
    let key = 7_003_u64;
    let (decided, requested) = (input(before + 1), input(before + 2));

    // The handler decides on `decided`; the caller-built idempotency binding
    // names `requested`, so the program and the key describe different intents.
    let program = candidate!(runtime, branch, SetRetentionBinding, key, decided);
    let outcome = application.compare_and_commit_program_action(
        program,
        &identities::<SetRetentionBinding>(&key, &requested),
        std::convert::identity,
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("a program must not commit under another input's idempotency binding");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::MutationInputMismatch
    );
    assert_eq!(
        read_retention(runtime, branch),
        before,
        "nothing was written"
    );

    let program = candidate!(runtime, branch, SetRetentionBinding, key, decided);
    let outcome = application.compare_and_commit_program_action(
        program,
        &identities::<SetRetentionBinding>(&key, &decided),
        std::convert::identity,
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the same input under its own idempotency binding commits: {outcome:?}"
    );
    assert_eq!(read_retention(runtime, branch), before + 1);
}
