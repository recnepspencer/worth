//! A stopped native traversal keeps its exact previous page for a fresh retry.
use super::*;
use crate::http::server::mutation_construction::correction::correction_denial;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[tokio::test]
async fn failed_native_history_resume_retains_cursor_and_revocation_ends_selection() {
    let fixture = fixture().await;
    let owner = BankHttpAuthenticatedOwner::from_principal(&fixture.principal);
    let shared = Arc::new(Mutex::new(BankHttpContinuationRegistry::new(
        1,
        Duration::from_secs(60),
    )));
    let first = access(&shared, |registry| {
        registry.register_correction_initial(
            owner.clone(),
            fixture.source.clone(),
            "first".into(),
            fixture.page.clone(),
        )
    })
    .await
    .unwrap_or_else(|_| panic!("native history slot"));
    let token = token(&first);
    let HistoryResumeAdmission::Execute(previous) = access(&shared, |registry| {
        registry.begin_correction_resume(&owner, &fixture.source, "canceled", &token)
    })
    .await
    else {
        panic!("native cursor admission");
    };
    assert!(
        matches!(access(&shared, |registry| registry.correction_page(&owner, &token)).await,
        Err(cause) if cause.kind == BankHttpDenialKind::Saturated)
    );
    let request_scope = scope();
    let request = fixture
        .application
        .runtime
        .request(&fixture.principal, &request_scope);
    let source = request
        .corrections()
        .select_coordinates(&fixture.source.clone().into(), limits())
        .unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped_scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(30)),
        cancellation.token(),
    );
    let stopped = fixture
        .application
        .runtime
        .request(&fixture.principal, &stopped_scope)
        .corrections()
        .continue_alternatives(
            &source,
            limits(),
            std::num::NonZeroUsize::MIN,
            previous.continuation().unwrap(),
        );
    let cause = correction_denial(
        stopped
            .err()
            .expect("canceled native continuation must stop"),
    );
    assert_eq!(cause.kind, BankHttpDenialKind::Cancelled);
    let returned = access(&shared, |registry| {
        registry.fail_correction_resume(&token, "canceled", cause, previous, true)
    })
    .await;
    assert!(
        returned.is_none(),
        "the original page is retained, not discarded after a retryable stop"
    );
    assert!(
        matches!(access(&shared, |registry| registry.begin_correction_resume(&owner, &fixture.source, "canceled", &token)).await,
        HistoryResumeAdmission::Replay(BankHttpCorrectionHistoryOutcome::Denied { denial, .. }) if denial.kind == BankHttpDenialKind::Cancelled)
    );
    let HistoryResumeAdmission::Execute(previous) = access(&shared, |registry| {
        registry.begin_correction_resume(&owner, &fixture.source, "retry", &token)
    })
    .await
    else {
        panic!("fresh request retries the same cursor");
    };
    assert!(Arc::ptr_eq(&previous, &fixture.page));
    let page = Arc::new(
        request
            .corrections()
            .continue_alternatives(
                &source,
                limits(),
                std::num::NonZeroUsize::MIN,
                previous.continuation().unwrap(),
            )
            .unwrap()
            .into_page(),
    );
    drop(previous);
    let completed = access(&shared, |registry| {
        registry.complete_correction_resume(&token, "retry", page)
    })
    .await
    .unwrap_or_else(|_| panic!("successor page is retained"));
    assert!(matches!(
        completed,
        BankHttpCorrectionHistoryOutcome::Delivered { complete: true, .. }
    ));
    access(&shared, |registry| {
        registry.deny_correction_replay(
            &token,
            crate::http::server::mutation_refusal::denial(BankHttpDenialKind::PermissionDenied),
        )
    })
    .await;
    assert!(
        matches!(access(&shared, |registry| registry.correction_page(&owner, &token)).await,
        Err(cause) if cause.kind == BankHttpDenialKind::Stale)
    );
    assert!(
        matches!(access(&shared, |registry| registry.begin_correction_resume(&owner, &fixture.source, "after-revocation", &token)).await,
        HistoryResumeAdmission::Denied(cause) if cause.kind == BankHttpDenialKind::Stale)
    );
    assert!(
        matches!(access(&shared, |registry| registry.replay_correction_initial(&owner, &fixture.source, "first")).await.unwrap().1,
        BankHttpCorrectionHistoryOutcome::Denied { denial, .. } if denial.kind == BankHttpDenialKind::PermissionDenied)
    );
}
