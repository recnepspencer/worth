//! Same genuine Source and probe have distinct candidate and exact meanings.
use super::super::{authoring::Edit, refinement};
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
#[test]
fn exact_refinement_prefix_agrees_with_native_and_routes_candidate_stable_changes() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::exact_refinement_prefix_agrees_with_native_and_routes_candidate_stable_changes"
        ),
    );
}
fn exact() -> refinement::FindExact {
    refinement::FindExact {
        directory: "A".into(),
        name: "sha".into(),
    }
}
fn run() {
    let calls = Arc::new(prefix_observation::Calls::default());
    let world = host::install_watch(calls.clone(), Arc::new(rule::Calls::new(calls.clone())));
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
    let (candidate, mut broad) = request
        .query(find("A", "sha"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(
        values(candidate.rows()),
        vec![("x".into(), "shared".into())]
    );
    let (refined, mut precise) = request
        .query(exact())
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    assert!(
        refined.rows().is_empty(),
        "the genuine candidate shared is not exact sha"
    );
    assert!(
        matches!(request.query(exact()).next_watch(&mut broad), Err(_)),
        "equal probes do not identify the same Parent/mode watch"
    );
    let equal = request
        .advanced()
        .mutate(rename("sha", "shared"))
        .expect_result_set(candidate.result_set_observation().unwrap().clone())
        .idempotency(&0x9199_0ffeu64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            equal,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "fresh same-value action must execute lawfully: {equal:?}"
    );
    match request
        .query(find("A", "sha"))
        .next_watch(&mut broad)
        .unwrap()
    {
        WorthQueryManagedIndexWatchOutcome::Pending => (),
        WorthQueryManagedIndexWatchOutcome::Delivered(update) => assert_eq!(
            values(update.read().rows()),
            vec![("x".into(), "shared".into())]
        ),
        _ => panic!("same-value update must preserve supported watch availability"),
    }
    assert!(matches!(
        request
            .query(find("A", "sha"))
            .next_watch(&mut broad)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    assert!(matches!(
        request.query(exact()).next_watch(&mut precise).unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    assert_eq!(
        values(request.query(find("A", "sha")).execute().unwrap().rows()),
        vec![("x".into(), "shared".into())]
    );
    assert!(request.query(exact()).execute().unwrap().rows().is_empty());
    for (offset, name) in ["sha", "shared"].into_iter().enumerate() {
        let changed = request
            .advanced()
            .mutate(rename("sha", name))
            .expect_result_set(
                request
                    .query(find("A", "sha"))
                    .execute()
                    .unwrap()
                    .into_result_set_observation()
                    .unwrap(),
            )
            .idempotency(&(0x9199_1000 + offset as u64))
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                changed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{changed:?}"
        );
        let WorthQueryManagedIndexWatchOutcome::Delivered(candidate) = request
            .query(find("A", "sha"))
            .next_watch(&mut broad)
            .unwrap()
        else {
            panic!("candidate output changed")
        };
        assert_eq!(candidate.read().rows()[0].name, name);
        let WorthQueryManagedIndexWatchOutcome::Delivered(refined) =
            request.query(exact()).next_watch(&mut precise).unwrap()
        else {
            panic!("exact truth changed inside unchanged candidate membership")
        };
        assert_eq!(refined.read().rows().len(), usize::from(name == "sha"));
    }
    // The exact Native rule admits distinct sha-prefixed names, then denies
    // two true exact names through the same real Source/refiner path.
    for (offset, name, commits) in [(2, "sha-other", true), (3, "sha", true)].into_iter() {
        let changed = request
            .advanced()
            .mutate(Edit::Item {
                directory: "A".into(),
                identity: format!("n{offset}"),
                name: name.into(),
            })
            .idempotency(&(0x9199_1000 + offset))
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert_eq!(
            matches!(
                changed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            commits,
            "{changed:?}"
        );
        if offset == 2 {
            let WorthQueryManagedIndexWatchOutcome::Delivered(candidate) = request
                .query(find("A", "sha"))
                .next_watch(&mut broad)
                .unwrap()
            else {
                panic!("new sha-other candidate must route to candidate mode")
            };
            assert_eq!(candidate.read().rows().len(), 2);
            assert!(
                matches!(
                    request.query(exact()).next_watch(&mut precise).unwrap(),
                    WorthQueryManagedIndexWatchOutcome::Pending
                ),
                "an exact-rejected entrant must not wake the exact-mode posting"
            );
        }
    }
    let before_denial = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let denied = request
        .advanced()
        .mutate(rename("sha", "sha"))
        .expect_result_set(
            request
                .query(find("A", "sha"))
                .execute()
                .unwrap()
                .into_result_set_observation()
                .unwrap(),
        )
        .idempotency(&0x9199_1004u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = denied
    else {
        panic!("exact Native rule must refuse: {denied:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let Some(WorthQueryCustomInvariantDenial::Violation { identity, .. }) =
        denial.custom_invariant_denial()
    else {
        panic!("Source/read/setup failure cannot counterfeit exact violation: {denial:?}")
    };
    assert_eq!(identity.rule_id.as_str(), "AtMostOneExactShaName");
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before_denial
    );
    let witness = denial
        .decode_invariant_refusal::<refinement::rule::ExactNameRefusalBinding>()
        .unwrap();
    assert_eq!(witness.committed,Some(1),"the same refusing Native scope has one exact sha amid nonexact sha-prefixed predecessor names");
    assert_eq!(witness.proposed, 2);
    let current = request.query(exact()).execute().unwrap();
    assert_eq!(current.rows().len(), 1);
    assert_eq!(current.rows()[0].identity, "n3");
    calls.reject_refinement.store(true, Ordering::SeqCst);
    let refused = request
        .query(refinement::FindExact {
            directory: "A".into(),
            name: "no-candidate".into(),
        })
        .execute()
        .err()
        .expect("empty candidates still validate exact probe");
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(refused) = refused else {
        panic!("exact provider refusal required: {refused:?}")
    };
    let WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Provider(cause)) =
        refused.cause()
    else {
        panic!("original domain refusal required: {refused:?}")
    };
    assert_eq!(
        cause.downcast_ref::<worth_query_consumer_values::PrefixIndexDenial>(),
        Some(&worth_query_consumer_values::PrefixIndexDenial::InvalidTopology)
    );
    calls.reject_refinement.store(false, Ordering::SeqCst);
    let cancellation = WorthQueryCancellationSource::new();
    let interrupted = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let fired = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observe = fired.clone();
    *calls.after_refinement.lock().unwrap() = Some(Box::new(move || {
        observe.store(true, Ordering::SeqCst);
        cancellation.cancel();
    }));
    let stopped = world
        .runtime()
        .request(&principal, &interrupted)
        .on_branch(world.current_world())
        .query(exact())
        .execute()
        .err()
        .expect("late exact cancellation must refuse");
    assert!(
        fired.load(Ordering::SeqCst),
        "the genuine row callback ran before refusal"
    );
    assert!(
        matches!(stopped,WorthQueryApplicationRequestQueryDenial::ManagedIndex(ref cause)
        if matches!(cause.cause(),WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Controlled(WorthQueryManagedIndexStop::Request(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)))))),
        "{stopped:?}"
    );
    assert_eq!(request.query(exact()).execute().unwrap().rows().len(), 1);
}

#[test]
fn exact_refinement_requires_the_declared_executable_at_installation() {
    lifecycle::isolated_allocation_case(
        installation,
        concat!(
            module_path!(),
            "::exact_refinement_requires_the_declared_executable_at_installation"
        ),
    );
}
fn installation() {
    use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
    let calls = Arc::new(prefix_observation::Calls::default());
    calls.omit_refinement.store(true, Ordering::SeqCst);
    let denied =
        host::try_install_unpopulated(calls.clone(), Arc::new(rule::Calls::new(calls.clone())))
            .err()
            .expect("declared refined Parent needs its executable");
    let WorthQueryInMemoryApplicationDenial::Contributions(cause) = denied else {
        panic!("{denied:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::MissingManagedIndexRefinement
    );
    assert_eq!(cause.subject(), "catalog_prefix_index");
    calls.omit_refinement.store(false, Ordering::SeqCst);
    assert!(
        host::try_install_unpopulated(calls.clone(), Arc::new(rule::Calls::new(calls))).is_ok(),
        "same schema and actual configured provider install lawfully"
    );
}
