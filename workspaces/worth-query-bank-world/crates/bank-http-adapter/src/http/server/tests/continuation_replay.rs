use std::sync::Arc;
use std::time::Duration;

use bank_domain::model::{AccountId, BankPrincipalId, CustomerRole};
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{GrantAccountAuthorization, RevokeAccountAuthorization};
use bank_server::{mutations, queries, BankMutationControls, BankReadControls};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance;

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
            Some(std::time::Instant::now() + Duration::from_secs(5)),
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
        .retain_operation()
        .unwrap_or_else(|cause| panic!("grant construction: {cause:?}"));
    let scope = fresh_scope();
    {
        let request = application.runtime.request(&owner, &scope);
        let WorthQuerySourceOnlyAdvance::Completed(granted) = granted.advance(&request) else {
            panic!("native grant completion")
        };
        assert!(granted.close(&request).is_ok());
    }

    let (server, _mutation_continuation) = bind_application(
        application.clone(),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind")
    .into_parts();
    let server = server.activate();
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
        "protocol": "v4",
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
        .controls(BankReadControls::current(fresh_scope(), 16).unwrap())
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
        .retain_operation()
        .unwrap_or_else(|cause| panic!("revoke construction: {cause:?}"));
    let scope = fresh_scope();
    {
        let request = application.runtime.request(&owner, &scope);
        let WorthQuerySourceOnlyAdvance::Completed(revoked) = revoked.advance(&request) else {
            panic!("native revoke completion")
        };
        assert!(revoked.close(&request).is_ok());
    }

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
    assert!(_mutation_continuation.finish().await.is_ok());
}

#[tokio::test]
async fn replay_revalidation_keeps_the_cache_on_retry_and_scrubs_it_on_drift() {
    use super::super::super::protocol::{BankHttpDenial, BankHttpNextAction};
    use super::super::continuation_executor::replay_fault::fail_next_replay_read;

    let account = AccountId::new(100).unwrap();
    let application = Arc::new(application(account));
    let (server, _mutation_continuation) =
        bind_application(application, BankHttpServerConfiguration::local_ephemeral())
            .await
            .expect("HTTP server should bind")
            .into_parts();
    let server = server.activate();
    let client = reqwest::Client::new();
    let endpoint = format!(
        "http://{}/v1/queries/account-activity/page",
        server.local_address()
    );
    let request = page_request(account, "replay-revalidation-fault");
    let first = post_page(&client, &endpoint, &request).await;
    let _token = continuation(&first);
    let denied = |kind| {
        move |outcome: &BankHttpAccountActivityPageOutcome| {
            matches!(
                outcome,
                BankHttpAccountActivityPageOutcome::Denied { denial, .. } if denial.kind == kind
            )
        }
    };

    // A temporary revalidation failure refuses this replay and keeps the
    // cached publication for the next one.
    fail_next_replay_read(
        "replay-revalidation-fault",
        BankHttpDenial::new(
            BankHttpDenialKind::ResourceExhausted,
            BankHttpNextAction::Retry,
        ),
    );
    let refused = post_page(&client, &endpoint, &request).await;
    assert!(
        denied(BankHttpDenialKind::ResourceExhausted)(&refused),
        "{refused:?}"
    );
    assert_eq!(post_page(&client, &endpoint, &request).await, first);

    // A drifted basis refuses the replay and retires the cached response.
    fail_next_replay_read(
        "replay-revalidation-fault",
        BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
    );
    for _ in 0..2 {
        let replay = post_page(&client, &endpoint, &request).await;
        assert!(denied(BankHttpDenialKind::Stale)(&replay), "{replay:?}");
    }
    server.shutdown().await.expect("server should shut down");
    assert!(_mutation_continuation.finish().await.is_ok());
}

#[tokio::test]
async fn abandoned_resume_finishes_and_replays_its_exact_response() {
    let account = AccountId::new(100).unwrap();
    let (application, gate) = held_authentication_application(account);
    let (server, _mutation_continuation) = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind")
    .into_parts();
    let server = server.activate();
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
        "protocol": "v4", "request_id": "abandoned-resume",
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
    assert!(_mutation_continuation.finish().await.is_ok());
}
