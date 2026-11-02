//! Exact observation consumers retain rejected candidate inputs and original stops.
use super::super::super::refinement::{self, mutation::RenameExact};
use super::*;
use worth_query_consumer_values::PrefixIndexDenial;

#[test]
fn realized_footprint_exact_rejected_candidate_inputs_remain_influence() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let exact = || refinement::FindExact {
            directory: "A".into(),
            name: "sha".into(),
        };
        let original = request.query(exact()).execute().unwrap();
        assert!(original.rows().is_empty()); // shared is a genuine rejected sha candidate.
        assert_eq!(
            request
                .query(find("A", "sha"))
                .execute()
                .unwrap()
                .rows()
                .len(),
            1
        );
        let source = original.into_result_set_observation().unwrap();
        assert!(source.has_verified_realized_footprint());
        insert(
            &world,
            &principal,
            &scope,
            "irrelevant",
            "zz-other",
            0x919a_7500,
        );
        let result = request
            .advanced()
            .mutate(RenameExact(rename("sha", "shared")))
            .expect_result_set(source)
            .idempotency(&0x919a_7501u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{:?}",
            result.commit_outcome()
        );
        let source = request
            .query(exact())
            .execute()
            .unwrap()
            .into_result_set_observation()
            .unwrap();
        let changed = request
            .advanced()
            .mutate(rename("sha", "sham"))
            .expect_result_set(
                request
                    .query(find("A", "sha"))
                    .execute()
                    .unwrap()
                    .into_result_set_observation()
                    .unwrap(),
            )
            .idempotency(&0x919a_7502u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        assert!(request.query(exact()).execute().unwrap().rows().is_empty());
        assert_eq!(
            request
                .query(find("A", "sha"))
                .execute()
                .unwrap()
                .rows()
                .len(),
            1
        );
        let head = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let refused = request
            .advanced()
            .mutate(RenameExact(rename("sha", "wrong")))
            .expect_result_set(source)
            .idempotency(&0x919a_7503u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
        let Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Attempt(cause),
        )) = refused
        else {
            panic!("unchanged empty exact DTO cannot erase a changed rejected candidate");
        };
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationAttemptDenialKind::SourceChanged
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &head
        );
    });
}

#[test]
fn realized_footprint_target_refinement_keeps_domain_failure_and_original_cancellation() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls.clone())));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let exact = || refinement::FindExact {
            directory: "A".into(),
            name: "sha".into(),
        };
        let source = request
            .query(exact())
            .execute()
            .unwrap()
            .into_result_set_observation()
            .unwrap();
        assert!(source.has_verified_realized_footprint());
        insert(
            &world,
            &principal,
            &scope,
            "irrelevant",
            "zz-other",
            0x919a_7600,
        );
        let head = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        calls.reject_refinement.store(true, Ordering::SeqCst);
        let refused = request
            .advanced()
            .mutate(RenameExact(rename("sha", "wrong")))
            .expect_result_set(source.clone())
            .idempotency(&0x919a_7601u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
        let Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Attempt(cause),
        )) = refused
        else {
            panic!("target domain denial cannot turn into success or stale membership");
        };
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationAttemptDenialKind::ManagedIndexSourceDenied
        );
        let Some(WorthQueryManagedIndexWorkDenial::Provider(provider)) =
            cause.managed_index_denial()
        else {
            panic!("target verification lost the concrete provider denial");
        };
        assert_eq!(
            provider.downcast_ref::<PrefixIndexDenial>(),
            Some(&PrefixIndexDenial::InvalidTopology)
        );
        calls.reject_refinement.store(false, Ordering::SeqCst);
        let cancellation = WorthQueryCancellationSource::new();
        let stopped = WorthQueryRequestScope::new(None, cancellation.token());
        let fired = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let observed = fired.clone();
        *calls.after_refinement.lock().unwrap() = Some(Box::new(move || {
            observed.store(true, Ordering::SeqCst);
            cancellation.cancel();
        }));
        let refused = world
            .runtime()
            .request(&principal, &stopped)
            .on_branch(world.current_world())
            .advanced()
            .mutate(RenameExact(rename("sha", "wrong")))
            .expect_result_set(source)
            .idempotency(&0x919a_7602u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
        assert!(
            fired.load(Ordering::SeqCst),
            "the real target refinement callback must cancel the original request"
        );
        let Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Attempt(cause),
        )) = refused
        else {
            panic!("target cancellation must retain original request refusal");
        };
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationAttemptDenialKind::Cancelled
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &head
        );
    });
}
