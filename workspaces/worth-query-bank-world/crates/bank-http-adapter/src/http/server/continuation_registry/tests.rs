//! Continuation inventory tests retain actual native activity custody.
use super::*;
use crate::http::{protocol::*, server::authenticated_owner::BankHttpAuthenticatedOwner};
use bank_domain::model::AccountId;
use bank_server::BankReadControls;
mod fixture;
use fixture::{fixture, scope, Fixture};

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
