pub(crate) use worth_query_testkit_certification_canonical::facade::certification::application_graph::canonical_admission::continuation::*;

#[test]
fn continuation_nested_decode_refusal_never_returns_a_page_or_successor() {
    let host = host::install_page(super::FUNDED_WORK);
    assert_funded_pages(&host);
    for name in ["root", "child"] {
        assert_eq!(host::read(&host, name).value, super::value());
    }
    query::reset(None, false);
    opaque::reset();
    drop(first_page(&host, &request_scope()).expect("the funded first page reaches both seeds"));
    let work = host
        .second_decode_work
        .get()
        .expect("funded page reached its second seed");
    drop(host);

    let mut host = host::install_page(super::FUNDED_WORK);
    for name in ["root", "child"] {
        assert_eq!(host::read(&host, name).value, super::value());
    }
    host.request_work = work;
    for name in ["root", "child"] {
        assert_eq!(host::read(&host, name).value, super::value());
    }
    query::reset(None, false);
    opaque::reset();
    let denial = match first_page(&host, &request_scope()) {
        Err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution(denial)) => denial,
        Err(other) => panic!("the admitted page must fail at decode completion: {other:?}"),
        Ok(_) => panic!("swallowed decode refusal must return neither page nor successor"),
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationContinuationDenialKind::Projection(
            WorthQueryApplicationProjectionDenialKind::ValueDecode(
                WorthQueryValueDecodeDenial::Checkpoint(
                    worth_execution::MapKernelStop::WorkCeiling
                )
            )
        )
    );
    assert_eq!(
        query::returned(),
        1,
        "the actual projector returned apparent success"
    );
    assert!(
        opaque::ignored_refusals() > 0,
        "the real nested seed swallowed native refusal"
    );
    let checkpoint = host::capture(&host);
    drop(host);
    let fresh = host::restore(checkpoint, super::FUNDED_WORK);
    assert_funded_pages(&fresh);
}

#[test]
fn continuation_late_cancellation_refuses_the_page_and_fresh_request_succeeds() {
    let host = host::install_page(super::FUNDED_WORK);
    // Both apparent success and a domain rejection must preserve cancellation
    // of the ORIGINAL request after the actual nondefault value was decoded.
    for reject in [false, true] {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(30)),
            cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
        query::reset(Some(cancellation), reject);
        opaque::reset();
        let denial = match first_page(&host, &scope) {
            Err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution(denial)) => denial,
            Err(other) => panic!("late cancellation must veto the admitted page: {other:?}"),
            Ok(_) => panic!("late cancellation must return neither page nor successor"),
        };
        assert_eq!(
            query::returned(),
            1,
            "cancellation follows actual decoded projection"
        );
        assert_eq!(opaque::ignored_refusals(), 0);
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationContinuationDenialKind::Projection(
                WorthQueryApplicationProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Scope(
                        worth_execution::WorkCeilingDenial::Stopped(
                            worth_execution::MapKernelStop::Cancelled
                        )
                    )
                )
            )
        );
    }
    // With a live original request, an actual application domain rejection
    // still propagates as such; cancellation is not a manufactured error map.
    query::reset(None, true);
    let denial = match first_page(&host, &request_scope()) {
        Err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution(denial)) => denial,
        Err(other) => panic!("the domain rejection must remain a projection denial: {other:?}"),
        Ok(_) => panic!("a rejecting projector must not publish a page or successor"),
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationContinuationDenialKind::Projection(
            WorthQueryApplicationProjectionDenialKind::DomainProjectionRejected,
        )
    );
    assert_funded_pages(&host);
}
