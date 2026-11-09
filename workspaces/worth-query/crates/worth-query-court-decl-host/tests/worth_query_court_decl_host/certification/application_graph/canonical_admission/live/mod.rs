pub(crate) use worth_query_testkit_certification_canonical::facade::certification::application_graph::canonical_admission::live::*;

#[test]
fn live_nested_decode_requires_original_funding_before_receipt_and_cause_handoff() {
    let funded = host::install(super::FUNDED_WORK);
    let mut live = open(&funded);
    publish(&funded, 78_001);
    let scope = request_scope();
    let principal = authenticate_operator(funded.program.installed_schema(), &scope);
    let request = funded
        .program
        .runtime()
        .request(&principal, &scope)
        .on_branch(funded.program.current_world());
    reset();
    let WorthQueryApplicationLiveOutcome::Delivered(update) = live.next(&request).unwrap() else {
        panic!("funded actual retained cause must deliver its nondefault nested value");
    };
    assert_eq!(update.result().value, super::value());
    assert_eq!(update.result().counter, 0);
    assert_eq!(returned(), 1);
    assert_eq!(opaque::ignored_refusals(), 0);
    assert!(matches!(
        live.next(&request).unwrap(),
        WorthQueryApplicationLiveOutcome::Pending
    ));
    assert!(matches!(
        live.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
    drop(funded);

    let narrow = host::install(super::FUNDED_WORK);
    let mut live = native::open(&narrow);
    publish(&narrow, 78_002);
    reset();
    let (primed, _) = native::next(&narrow, &mut live, super::FUNDED_WORK);
    assert!(matches!(
        primed,
        WorthQueryApplicationLiveOutcome::Delivered(_)
    ));
    publish(&narrow, 78_005);
    reset();
    let (baseline, _) = native::next(&narrow, &mut live, super::FUNDED_WORK);
    let WorthQueryApplicationLiveOutcome::Delivered(update) = baseline else {
        panic!("the same native delivery helper must genuinely deliver before narrowing");
    };
    assert_eq!(update.result().value, super::value());
    assert_eq!(returned(), 1);
    assert_eq!(opaque::ignored_refusals(), 0);
    // Use the actual funded second seed's accepted consult prefix. Sharing
    // removes copy work; wire-length subtraction cannot locate this boundary.
    let work = narrow
        .second_decode_work
        .get()
        .expect("the funded native live owner reached its second decoder seed");
    publish(&narrow, 78_003);
    reset();
    let (denied, _) = native::next(&narrow, &mut live, work);
    let WorthQueryApplicationLiveOutcome::ProjectionDenied(denial) = denied else {
        panic!("the swallowed second admitted decode must not hand out a live result or receipt");
    };
    assert_eq!(
        denial,
        WorthQueryApplicationProjectionDenialKind::ValueDecode(
            WorthQueryValueDecodeDenial::Checkpoint(worth_execution::MapKernelStop::WorkCeiling),
        )
    );
    assert_eq!(
        returned(),
        1,
        "the actual live projector caught the error and returned success"
    );
    assert!(
        opaque::ignored_refusals() > 0,
        "the actual nested seed swallowed native admission refusal"
    );
    assert!(matches!(
        native::next(&narrow, &mut live, super::FUNDED_WORK).0,
        WorthQueryApplicationLiveOutcome::Closed
    ));
    drop(live);
    assert_eq!(
        host::read(&narrow, "root").value,
        super::value(),
        "a fresh affordable request has no latched predecessor refusal"
    );
    let mut fresh = native::open(&narrow);
    publish(&narrow, 78_004);
    reset();
    let (retried, _) = native::next(&narrow, &mut fresh, super::FUNDED_WORK);
    let WorthQueryApplicationLiveOutcome::Delivered(update) = retried else {
        panic!("a fresh funded native live request must not inherit the swallowed refusal");
    };
    assert_eq!(update.result().value, super::value());
    assert_eq!(returned(), 1);
    assert_eq!(opaque::ignored_refusals(), 0);
    assert!(matches!(
        fresh.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}

#[test]
fn live_late_original_cancellation_cannot_ack_and_a_fresh_live_request_succeeds() {
    let host = host::install(super::FUNDED_WORK);
    let mut live = open(&host);
    publish(&host, 78_101);
    let cancellation = WorthQueryCancellationSource::new();
    let original_scope = request_scope();
    let scope = WorthQueryRequestScope::new(
        original_scope.deadline().map(|deadline| {
            deadline
                .instant_on(original_scope.clock())
                .expect("the request deadline retains its clock")
        }),
        cancellation.token(),
        original_scope.clock().clone(),
    );
    let principal = authenticate_operator(host.program.installed_schema(), &scope);
    let request = host
        .program
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.program.current_world());
    let guard = CancelAfterProjection::arm(cancellation);
    let outcome = live.next(&request).unwrap();
    assert_eq!(
        returned(),
        1,
        "cancellation lands after real nondefault decoding"
    );
    assert_eq!(opaque::ignored_refusals(), 0);
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationLiveOutcome::ProjectionDenied(
                WorthQueryApplicationProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Scope(
                        worth_execution::WorkCeilingDenial::Stopped(
                            worth_execution::MapKernelStop::Cancelled
                        )
                    )
                )
            ) | WorthQueryApplicationLiveOutcome::Cancelled
        ),
        "a late original cancellation may not produce a delivery/receipt"
    );
    assert!(matches!(
        live.next(&request).unwrap(),
        WorthQueryApplicationLiveOutcome::Closed
    ));
    drop(guard);
    drop(live);

    let mut fresh = open(&host);
    publish(&host, 78_102);
    let scope = request_scope();
    let principal = authenticate_operator(host.program.installed_schema(), &scope);
    let request = host
        .program
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.program.current_world());
    reset();
    let WorthQueryApplicationLiveOutcome::Delivered(update) = fresh.next(&request).unwrap() else {
        panic!("a fresh funded original request must deliver through the same live owner");
    };
    assert_eq!(update.result().value, super::value());
    assert_eq!(returned(), 1);
    assert!(matches!(
        fresh.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}
