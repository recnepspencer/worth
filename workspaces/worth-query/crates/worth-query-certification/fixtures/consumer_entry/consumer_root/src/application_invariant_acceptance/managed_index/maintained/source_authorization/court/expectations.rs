//! The generic Rename handler never performs a fresh protected Source read.
use super::*;
#[test]
fn native_source_expectation_retains_its_independent_read_policy() {
    lifecycle::run(|| {
        let setup = Setup::new();
        let request = setup
            .world
            .runtime()
            .request(&setup.principal, &setup.scope)
            .on_branch(setup.world.current_world());
        let original = request
            .query(native())
            .execute()
            .unwrap()
            .into_result_set_observation()
            .unwrap();
        setup.set_readable(false, 0x919c_0030);
        let before = callbacks(&setup.calls.native);
        let head = request.retain_current_read().unwrap();
        let result = request
            .advanced()
            .mutate(expectation::RenameNative(Setup::rename()))
            .expect_result_set(original)
            .idempotency(&0x919c_0032u64)
            .execute_in_program(&setup.world, ExecutionAllocationPolicy::SystemAllocation);
        source_authorization_lost(result);
        assert_eq!(
            callbacks(&setup.calls.native),
            before,
            "the handler never freshly queries Source"
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            head.selected_commit()
        );
        setup.set_readable(true, 0x919c_0031);
        let fresh = request
            .query(native())
            .execute()
            .unwrap()
            .into_result_set_observation()
            .unwrap();
        let result = request
            .advanced()
            .mutate(expectation::RenameNative(Setup::rename()))
            .expect_result_set(fresh)
            .idempotency(&0x919c_0032u64)
            .execute_in_program(&setup.world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    });
}
#[test]
fn completed_source_expectation_retains_its_independent_read_policy() {
    lifecycle::run(|| {
        let setup = Setup::new();
        let request = setup
            .world
            .runtime()
            .request(&setup.principal, &setup.scope)
            .on_branch(setup.world.current_world());
        let index = request
            .query(completed())
            .open_index::<CompletedIndex>()
            .unwrap();
        request.query(completed()).rebuild_index(&index).unwrap();
        let original = request
            .query(completed())
            .execute()
            .unwrap()
            .into_result_set_observation()
            .unwrap();
        setup.set_readable(false, 0x919c_0040);
        let before = callbacks(&setup.calls.completed);
        let head = request.retain_current_read().unwrap();
        let result = request
            .advanced()
            .mutate(expectation::RenameCompleted(Setup::rename()))
            .expect_result_set(original)
            .idempotency(&0x919c_0042u64)
            .execute_in_program(&setup.world, ExecutionAllocationPolicy::SystemAllocation);
        source_authorization_lost(result);
        assert_eq!(callbacks(&setup.calls.completed), before);
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            head.selected_commit()
        );
        setup.set_readable(true, 0x919c_0041);
        request.query(completed()).rebuild_index(&index).unwrap();
        let fresh = request
            .query(completed())
            .execute()
            .unwrap()
            .into_result_set_observation()
            .unwrap();
        let result = request
            .advanced()
            .mutate(expectation::RenameCompleted(Setup::rename()))
            .expect_result_set(fresh)
            .idempotency(&0x919c_0042u64)
            .execute_in_program(&setup.world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        index.close();
    });
}

#[test]
fn separately_admitted_source_reads_keep_semantic_identity_for_retained_replay() {
    lifecycle::run(|| {
        let setup = Setup::new();
        let request = setup
            .world
            .runtime()
            .request(&setup.principal, &setup.scope)
            .on_branch(setup.world.current_world());
        let first = request
            .query(native())
            .execute()
            .unwrap()
            .into_result_set_observation()
            .unwrap();
        let callbacks_before = callbacks(&setup.calls.native);
        // This is a second full public entry/Source permission admission, not a
        // cloned observation. Native/Bridge decision issuance ordinals differ.
        let second = request
            .query(native())
            .execute()
            .unwrap()
            .into_result_set_observation()
            .unwrap();
        assert!(callbacks(&setup.calls.native).0 > callbacks_before.0);
        let committed = request
            .advanced()
            .mutate(expectation::RenameNative(Setup::rename()))
            .expect_result_set(first)
            .idempotency(&0x919c_0060u64)
            .execute_in_program(&setup.world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            committed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        let head = request.retain_current_read().unwrap();
        let replayed = request
            .advanced()
            .mutate(expectation::RenameNative(Setup::rename()))
            .expect_result_set(second)
            .idempotency(&0x919c_0060u64)
            .execute_in_program(&setup.world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                replayed,
                WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
            ),
            "{replayed:?}"
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            head.selected_commit()
        );
    });
}

fn source_authorization_lost(
    result: Result<
        WorthQueryApplicationMutationOutcome<mutation::Denial, mutation::Renamed>,
        WorthQueryApplicationRequestMutationDenial,
    >,
) {
    let denial = result.expect_err("original Source READ must refuse before publication");
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = denial
    else {
        panic!("expected the retained Source authorization cause: {denial:?}");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied
    );
    assert_eq!(
        cause.request_authority().map(|cause| cause.kind()),
        Some(WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::StaleAuthorization,
        ),)
    );
}
