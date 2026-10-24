//! Shared inventory tests retain actual native history and activity continuations.
use super::*;
use crate::http::{
    protocol::*,
    server::{
        authenticated_owner::BankHttpAuthenticatedOwner,
        continuation_registry::correction_history::HistoryResumeAdmission,
    },
};
use bank_domain::model::AccountId;
use bank_server::BankReadControls;
mod failure_recovery;
mod fixture;
mod stale_resolution;
use fixture::{fixture, limits, scope, Fixture};

fn token(outcome: &BankHttpCorrectionHistoryOutcome) -> String {
    match outcome {
        BankHttpCorrectionHistoryOutcome::Delivered { continuation, .. } => continuation.clone(),
        _ => panic!("stored page"),
    }
}
fn activity(
    fixture: &Fixture,
) -> (
    BankHttpAccountActivity,
    Option<bank_server::BankAccountActivityContinuation>,
    BankHttpQueryPublication,
) {
    let page = fixture
        .application
        .runtime
        .account_activity(AccountId::new(100).unwrap())
        .as_principal(&fixture.principal)
        .page(BankReadControls::current(scope(), 1).unwrap())
        .unwrap();
    let (published, continuation) = page.into_parts();
    let activity = published
        .rows()
        .first()
        .map(BankHttpAccountActivity::from)
        .unwrap();
    let publication = crate::http::server::query_publication::describe_query_publication(
        published.receipt(),
        BankHttpQueryCapabilityPurpose::AccountActivityReview,
    );
    (activity, continuation, publication)
}

#[tokio::test]
async fn history_inventory_shares_capacity_and_replays_only_its_owner_source_and_purpose() {
    let fixture = fixture().await;
    let owner = BankHttpAuthenticatedOwner::from_principal(&fixture.principal);
    let mut registry = BankHttpContinuationRegistry::new(2, Duration::from_secs(60));
    let first = registry
        .register_correction_initial(
            owner.clone(),
            fixture.source.clone(),
            "initial".into(),
            fixture.page.clone(),
        )
        .unwrap_or_else(|_| panic!("history slot"));
    let token = token(&first);
    assert_eq!(
        registry
            .replay_correction_initial(&owner, &fixture.source, "initial")
            .unwrap()
            .1,
        first
    );
    let (rows, continuation, publication) = activity(&fixture);
    registry
        .register_initial(
            owner.clone(),
            AccountId::new(100).unwrap(),
            "activity".into(),
            continuation,
            rows,
            publication,
        )
        .unwrap_or_else(|_| panic!("activity slot"));
    assert!(registry
        .register_correction_initial(
            owner.clone(),
            fixture.source.clone(),
            "full".into(),
            fixture.page.clone()
        )
        .is_err());
    assert!(!registry.correction_initial_has_capacity());
    assert_eq!(
        registry
            .replay_correction_initial(&owner, &fixture.source, "initial")
            .unwrap()
            .1,
        first,
        "an existing initial response remains replayable when capacity is full"
    );
    let credential: BankHttpCredential = serde_json::from_value(serde_json::json!({"id_token":"test-only", "access_token":"estate-subject", "nonce":"test-only"})).unwrap();
    let other =
        crate::http::server::authentication::BankHttpApplicationAuthenticator::authenticate(
            &fixture.application,
            credential,
            &scope(),
        )
        .await
        .unwrap();
    assert!(
        matches!(registry.correction_page(&BankHttpAuthenticatedOwner::from_principal(&other), &token), Err(cause) if cause.kind == BankHttpDenialKind::PermissionDenied)
    );
    let activity_token = registry
        .replay_initial(&owner, AccountId::new(100).unwrap(), "activity")
        .unwrap()
        .0;
    assert!(
        matches!(registry.correction_page(&owner, &activity_token), Err(cause) if cause.kind == BankHttpDenialKind::Stale)
    );
    let mut foreign = fixture.source.clone();
    foreign.world_commit += 1;
    assert!(
        matches!(registry.begin_correction_resume(&owner, &foreign, "foreign", &token), HistoryResumeAdmission::Denied(cause) if cause.kind == BankHttpDenialKind::Stale)
    );
    if let Some(ContinuationRecord::Correction(record)) = registry.records.get_mut(&token) {
        record.expires_at = Instant::now();
    }
    assert!(
        matches!(registry.correction_page(&owner, &token), Err(cause) if cause.kind == BankHttpDenialKind::Stale)
    );
}

#[tokio::test]
async fn disappeared_history_inflight_completion_cannot_publish_a_phantom_token() {
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
    .unwrap_or_else(|_| panic!("native history"));
    let token = token(&first);
    let HistoryResumeAdmission::Execute(previous) = access(&shared, |registry| {
        registry.begin_correction_resume(&owner, &fixture.source, "resume", &token)
    })
    .await
    else {
        panic!("native continuation");
    };
    let next = {
        let request_scope = scope();
        let request = fixture
            .application
            .runtime
            .request(&fixture.principal, &request_scope);
        let source = request
            .corrections()
            .select_coordinates(&fixture.source.clone().into(), limits())
            .unwrap();
        Arc::new(
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
        )
    };
    drop(previous);
    access(&shared, |registry| {
        if let Some(record) = registry.records.remove(&token) {
            registry.retired.push(record);
        }
    })
    .await;
    let returned = access(&shared, |registry| {
        registry.complete_correction_resume(&token, "resume", next.clone())
    })
    .await;
    assert!(returned.is_err());
    assert!(access(&shared, |registry| registry.records.is_empty()).await);
    drop(returned); // Returned new native custody is released after access unlocked.
}

#[tokio::test]
async fn disappeared_activity_inflight_completion_cannot_publish_a_phantom_token() {
    let fixture = fixture().await;
    let owner = BankHttpAuthenticatedOwner::from_principal(&fixture.principal);
    let shared = Arc::new(Mutex::new(BankHttpContinuationRegistry::new(
        1,
        Duration::from_secs(60),
    )));
    let (rows, continuation, publication) = activity(&fixture);
    access(&shared, |registry| {
        registry.register_initial(
            owner.clone(),
            AccountId::new(100).unwrap(),
            "first".into(),
            continuation,
            rows,
            publication,
        )
    })
    .await
    .unwrap_or_else(|_| panic!("native activity"));
    let token = access(&shared, |registry| {
        registry
            .replay_initial(&owner, AccountId::new(100).unwrap(), "first")
            .unwrap()
            .0
    })
    .await;
    let ResumeAdmission::Execute(continuation) = access(&shared, |registry| {
        registry.begin_resume(&owner, AccountId::new(100).unwrap(), "resume", &token)
    })
    .await
    else {
        panic!("activity continuation");
    };
    let request_scope = scope();
    let page = fixture
        .application
        .runtime
        .account_activity(AccountId::new(100).unwrap())
        .as_principal(&fixture.principal)
        .resume(
            continuation,
            worth_query_host::facade::primary_graph::WorthQueryApplicationQueryResumeControls::new(
                std::num::NonZeroUsize::MIN,
                &request_scope,
            ),
        )
        .unwrap();
    let (published, continuation) = page.into_parts();
    let rows = BankHttpAccountActivity::from(&published.rows()[0]);
    let publication = crate::http::server::query_publication::describe_query_publication(
        published.receipt(),
        BankHttpQueryCapabilityPurpose::AccountActivityReview,
    );
    drop(published);
    access(&shared, |registry| {
        if let Some(record) = registry.records.remove(&token) {
            registry.retired.push(record);
        }
    })
    .await;
    let returned = access(&shared, |registry| {
        registry.complete_resume(&token, "resume".into(), rows, continuation, publication)
    })
    .await;
    assert!(returned.is_err());
    assert!(access(&shared, |registry| registry.records.is_empty()).await);
    drop(returned);
}
