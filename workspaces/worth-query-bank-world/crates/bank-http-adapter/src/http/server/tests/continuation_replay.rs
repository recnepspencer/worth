use std::sync::Arc;
use std::time::Duration;

use bank_domain::model::{AccountId, BankPrincipalId, CustomerRole};
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{GrantAccountAuthorization, RevokeAccountAuthorization};
use bank_server::{mutations, queries, BankMutationControls, BankReadControls};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

use super::fixture::held_authentication_application;
use super::{
    application, bind_application, continuation, controls_json, page_request, post_page,
    BankHttpAccountActivityPageOutcome, BankHttpDenialKind, BankHttpServerConfiguration,
};
#[tokio::test]
async fn revoked_viewer_cannot_replay_cached_activity_pages() {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;

    let account = AccountId::new(100).unwrap();
    let application = Arc::new(application(account));
    let fresh_scope = || {
        let cancellation = WorthQueryCancellationSource::new();
        WorthQueryRequestScope::new(
            std::time::Instant::now() + Duration::from_secs(5),
            cancellation.token(),
        )
    };
    let owner = application.authenticate(&fresh_scope()).await;
    let viewer_id = BankPrincipalId::new(2).unwrap();
    let granted = application
        .runtime
        .mutate(mutations::grant_account_access(GrantAccountAuthorization {
            account,
            principal: viewer_id,
            role: CustomerRole::Viewer,
        }))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            fresh_scope(),
            BankIdempotencyKey::new("http-replay-grant-viewer").unwrap(),
        ))
        .execute();
    assert!(matches!(
        granted,
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
    ));

    let server = bind_application(
        application.clone(),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let page_endpoint = format!(
        "http://{}/v1/queries/account-activity/page",
        server.local_address()
    );
    let resume_endpoint = format!(
        "http://{}/v1/queries/account-activity/resume",
        server.local_address()
    );
    let mut first_request = page_request(account, "revoked-replay-page");
    first_request["credential"]["access_token"] = serde_json::json!("estate-subject");
    let first = post_page(&client, &page_endpoint, &first_request).await;
    let _page_token = continuation(&first);
    let mut resume_origin = page_request(account, "revoked-replay-resume-origin");
    resume_origin["credential"]["access_token"] = serde_json::json!("estate-subject");
    let origin_page = post_page(&client, &page_endpoint, &resume_origin).await;
    let token = continuation(&origin_page).to_owned();
    let resume_request = serde_json::json!({
        "protocol": "v1",
        "request_id": "revoked-replay-resume",
        "credential": first_request["credential"],
        "controls": controls_json(1),
        "account": account.canonical_text(),
        "continuation": token
    });
    let resumed = post_page(&client, &resume_endpoint, &resume_request).await;
    assert!(matches!(
        resumed,
        BankHttpAccountActivityPageOutcome::Delivered { .. }
    ));

    let users = application
        .runtime
        .query(queries::account_authorized_users(account))
        .as_principal(&owner)
        .controls(BankReadControls::current(fresh_scope(), 16, 20_000).unwrap())
        .execute()
        .expect("owner should read account grants");
    let authorization = users.rows()[0]
        .users()
        .iter()
        .find(|user| user.principal() == viewer_id)
        .expect("viewer grant should exist")
        .authorization();
    let revoked = application
        .runtime
        .mutate(mutations::revoke_account_access(
            RevokeAccountAuthorization {
                account,
                authorization,
            },
        ))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            fresh_scope(),
            BankIdempotencyKey::new("http-replay-revoke-viewer").unwrap(),
        ))
        .execute();
    assert!(matches!(
        revoked,
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
    ));

    for (endpoint, request) in [
        (&page_endpoint, &first_request),
        (&resume_endpoint, &resume_request),
    ] {
        let replay = post_page(&client, endpoint, request).await;
        assert!(matches!(
            replay,
            BankHttpAccountActivityPageOutcome::Denied { denial, .. }
                if denial.kind == BankHttpDenialKind::PermissionDenied
        ));
    }
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn temporary_replay_read_failure_keeps_the_original_response() {
    let account = AccountId::new(100).unwrap();
    let application = Arc::new(application(account));
    let server = bind_application(application, BankHttpServerConfiguration::local_ephemeral())
        .await
        .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let endpoint = format!(
        "http://{}/v1/queries/account-activity/page",
        server.local_address()
    );
    let request = page_request(account, "temporary-replay-budget");
    let first = post_page(&client, &endpoint, &request).await;
    let _token = continuation(&first);

    let mut under_budget = request.clone();
    under_budget["controls"]["maximum_work"] = serde_json::json!(1);
    let denied = post_page(&client, &endpoint, &under_budget).await;
    assert!(matches!(
        denied,
        BankHttpAccountActivityPageOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::ResourceExhausted
    ));
    assert_eq!(
        post_page(&client, &endpoint, &request).await,
        first,
        "a temporary revalidation failure must not erase the cached publication"
    );
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn abandoned_resume_finishes_and_replays_its_exact_response() {
    let account = AccountId::new(100).unwrap();
    let (application, gate) = held_authentication_application(account);
    let server = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let page_endpoint = format!(
        "http://{}/v1/queries/account-activity/page",
        server.local_address()
    );
    let resume_endpoint = format!(
        "http://{}/v1/queries/account-activity/resume",
        server.local_address()
    );
    let page_request = page_request(account, "abandoned-resume-origin");
    let first = tokio::spawn({
        let client = client.clone();
        let endpoint = page_endpoint.clone();
        async move { post_page(&client, &endpoint, &page_request).await }
    });
    gate.wait_for_calls(1).await;
    gate.release(1);
    let first = first.await.expect("first page task should finish");
    let resume_request = serde_json::json!({
        "protocol": "v1", "request_id": "abandoned-resume",
        "credential": super::credential_json(), "controls": controls_json(1),
        "account": account.canonical_text(), "continuation": continuation(&first)
    });
    let abandoned = tokio::spawn({
        let client = client.clone();
        let endpoint = resume_endpoint.clone();
        let request = resume_request.clone();
        async move { post_page(&client, &endpoint, &request).await }
    });
    gate.wait_for_calls(2).await;
    abandoned.abort();
    let _ = abandoned.await;
    gate.release(1);

    let mut crossed = resume_request.clone();
    crossed["request_id"] = serde_json::json!("abandoned-resume-different-id");
    let different = tokio::spawn({
        let client = client.clone();
        let endpoint = resume_endpoint.clone();
        async move { post_page(&client, &endpoint, &crossed).await }
    });
    gate.wait_for_calls(3).await;
    gate.release(1);
    assert!(matches!(
        different.await.expect("different request should finish"),
        BankHttpAccountActivityPageOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::Stale
    ));
    let replay = tokio::spawn({
        let client = client.clone();
        let endpoint = resume_endpoint.clone();
        async move { post_page(&client, &endpoint, &resume_request).await }
    });
    gate.wait_for_calls(4).await;
    gate.release(1);
    assert!(matches!(
        replay.await.expect("same-id retry should finish"),
        BankHttpAccountActivityPageOutcome::Delivered {
            continuation: None,
            ..
        }
    ));
    server.shutdown().await.expect("server should shut down");
}
