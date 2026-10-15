//! Real registered-attempt interleave and exact native evidence; no fabricated outcome.
use super::*;

pub(super) fn assert_stale(
    outcome: &WorthQueryApplicationMutationOutcome<
        super::super::schema::SetRetentionDenial,
        super::super::schema::DocumentRetentionWritten,
    >,
) {
    let WorthQueryApplicationMutationOutcome::Commit(U::Denied(cause)) = outcome else {
        panic!("registered exact-basis loser must retain its actual invariant refusal: {outcome:?}")
    };
    assert_eq!(cause.kind(), Kind::ProductBasisStale);
    assert_eq!(cause.stage(), Stage::InvariantExecution);
    assert_eq!(
        cause.invariant_execution_failure().unwrap().kind(),
        InvariantKind::ProductBasisStale
    );
}

pub(super) fn stale_owner(
    host: &Host,
    competing_days: u64,
    expected_retention: Option<u64>,
) -> fixture::Operation {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let mut conditions = worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions::new().expect_fact(
        super::super::schema::DocumentIdentityField::reference(),
        worth_query_host::facade::declaration::application_schema::ApplicationEncodedScalarValue::try_new(super::super::schema::DOCUMENT_IDENTITY.to_owned()).unwrap(),
    );
    if let Some(days) = expected_retention {
        conditions = conditions.expect_fact(super::super::schema::DocumentRetentionField::reference(),
            worth_query_host::facade::declaration::application_schema::ApplicationEncodedScalarValue::try_new(days).unwrap());
    }
    let original = request
        .mutate(fixture::OriginalRetention {
            input: super::super::schema::SetRetentionInput {
                identity: super::super::schema::DOCUMENT_IDENTITY.to_owned(),
                retention_days: 19,
            },
        })
        .without_source()
        .preconditions(conditions)
        .idempotency_owned(0x9176_5019)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<fixture::SourceOnlyRetentionProgram, fixture::RetentionAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("original typed conditions construct: {cause:?}"));
    drop(request);
    drop(principal);
    drop(scope);
    let pause = host
        .runtime()
        .pause_after_application_attempt_registration_for_test(NonZeroUsize::new(1).unwrap());
    std::thread::scope(|threads| {
        let attempt = threads.spawn(|| {
            let scope = request_scope();
            let principal = authenticate_operator(host.installed_schema(), &scope);
            original.advance(&host.runtime().request(&principal, &scope))
        });
        assert!(
            pause.wait_until_reached(Duration::from_secs(10)),
            "real original attempt registered"
        );
        assert_eq!(host.contacts.counts(), (1, 1));
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let competitor = fixture::completed(
            fixture::operation(host, &request, competing_days, 0x9176_5021),
            &request,
        );
        competitor
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("competitor close: {cause:?}"));
        let winner = observe_head(host.runtime(), host.current_world());
        pause.release();
        let original = match attempt.join().unwrap() {
            WorthQuerySourceOnlyAdvance::Progress(operation) => operation,
            WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
                assert_stale(terminal.native_outcome().expect("native terminal evidence"));
                panic!("proved no-effect unpinned loser must retain a retryable owner")
            }
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
                panic!("unexpected preparation refusal: {denial:?}")
            }
            _ => panic!("original stale candidate cannot complete"),
        };
        assert_stale(original.native_retry_outcome().unwrap());
        assert_eq!(
            original.posture(),
            WorthQuerySourceOnlyPosture::RetryableNoEffect
        );
        assert_eq!(
            observe_head(host.runtime(), host.current_world()).selected_commit(),
            winner.selected_commit()
        );
        assert_eq!(
            read_retention(host.runtime(), host.current_world()),
            competing_days
        );
        assert_eq!(host.contacts.counts(), (2, 2));
        assert_eq!(host.runtime().active_application_attempts_for_test(), 0);
        assert!(host
            .runtime()
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .is_empty());
        original
    })
}
