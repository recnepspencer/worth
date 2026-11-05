use super::*;
#[test]
fn original_cancellation_after_candidate_build_prevents_publication() {
    let cancellation = WorthQueryCancellationSource::new();
    let host = world::install_with_control(Some(cancellation.clone()));
    let before = world::pin(&host);
    let scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(authoring::Edit {
            key: "record".into(),
            change: authoring::Change::Second,
        })
        .without_source()
        .idempotency(&"cancel-before-publish".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let Err(WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(denial),
    )) = result
    else {
        panic!("original cancellation must refuse: {result:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied
    );
    assert_eq!(denial.subject(), "Transition");
    assert_eq!(
        denial
            .request_authority()
            .expect("original control cause")
            .kind(),
        WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
            WorthQueryProviderSessionControlStopKind::Cancelled
        ))
    );
    assert_eq!(
        before.selected_commit(),
        world::pin(&host).selected_commit()
    );
    assert_eq!(
        world::read(&host),
        query::Row {
            class: 1,
            first: Some(11),
            second: None,
            required: None,
            third: None
        }
    );
}

#[test]
fn retirement_and_recovery_keep_the_record_absent() {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let result = request
        .advanced()
        .mutate(retirement::Retire {
            key: "record".into(),
        })
        .without_source()
        .idempotency(&"retire".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    assert_absent(&host);
    let recovered = world::restore(world::checkpoint(&host));
    assert_absent(&recovered);
}

fn assert_absent(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let denial = host
        .runtime()
        .request(&principal, &scope)
        .query(query::Read {
            key: "record".into(),
        })
        .execute()
        .err()
        .expect("retired record");
    let WorthQueryApplicationRequestQueryDenial::ScopeResolution(denial) = denial else {
        panic!("actual retired scope: {denial:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryEntityResolutionDenialKind::Refused(
            WorthQueryEntityResolutionRefusalKind::UnknownEntity
        )
    );
    assert_eq!(denial.subject(), "Key");
}

#[test]
fn original_expired_deadline_refuses_before_any_publication() {
    let host = world::install();
    let before = world::pin(&host);
    let fresh = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &fresh);
    let scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now()),
        WorthQueryCancellationSource::new().token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(authoring::Edit {
            key: "record".into(),
            change: authoring::Change::Second,
        })
        .without_source()
        .idempotency(&"expired".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(
        matches!(
            outcome,
            Err(
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::DeadlineExceeded
                    )
                )
            )
        ),
        "{outcome:?}"
    );
    assert_eq!(
        before.selected_commit(),
        world::pin(&host).selected_commit()
    );
}
