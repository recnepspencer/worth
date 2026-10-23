//! Native target identity chooses refresh for changed reads and refusal for foreign owners.
use super::*;
use crate::http::server::mutation_construction::correction::correction_denial;
use crate::http::server::{bind_application, BankHttpServerConfiguration};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationCorrectionDenial,
    primary_graph::{
        CompositeAncestorSelectionDenial, ProductBranchAncestorSelectionDenial,
        RuntimeWorldServiceDenial, WorthQueryCorrectionDerivationDenial,
        WorthQueryCorrectionSelectionDenial,
    },
};

#[tokio::test]
async fn advanced_history_page_requires_refresh_while_foreign_runtime_stays_denied() {
    let (retained, owners) = super::fixture::retained_fixture().await;
    let Fixture {
        application,
        principal,
        source,
        page,
    } = retained;
    drop(page);
    let application = Arc::new(application);
    let (server, continuation) = bind_application(
        Arc::clone(&application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    let client = reqwest::Client::new();
    let base = format!("http://{}", server.local_address());
    let completed: BankHttpCorrectionHistoryOutcome = post(
        &client,
        &base,
        "/v1/corrections/history",
        &history_request("complete", &source, 16),
    )
    .await;
    assert!(matches!(
        &completed,
        BankHttpCorrectionHistoryOutcome::Delivered { complete: true, .. }
    ));
    let completed_token = token(&completed);
    let partial: BankHttpCorrectionHistoryOutcome = post(
        &client,
        &base,
        "/v1/corrections/history",
        &history_request("partial", &source, 1),
    )
    .await;
    assert!(matches!(
        &partial,
        BankHttpCorrectionHistoryOutcome::Delivered {
            complete: false,
            ..
        }
    ));
    let partial_token = token(&partial);
    let before = balance(&client, &base, "before").await;
    let deposited = continuation.execute(serde_json::from_value(serde_json::json!({
        "protocol":"v4", "request_id":"advance", "credential":credential(), "command":"create",
        "custody_id":"advance", "sequence":1, "controls":{"deadline_milliseconds":5000,"maximum_history_work":128},
        "idempotency_key":"advance-history-page", "operation":{"operation":"deposit","institution":"fixture:1","account":"fixture:100","amount_minor_units":17}
    })).unwrap()).await;
    assert!(
        matches!(&deposited, BankHttpMutationOutcome::Completed { .. }),
        "ordinary branch advance: {deposited:?}"
    );
    let closed = continuation.execute(serde_json::from_value(serde_json::json!({
        "protocol":"v4", "request_id":"close-advance", "credential":credential(), "command":"close",
        "custody_id":"advance", "sequence":2, "controls":{"deadline_milliseconds":5000}
    })).unwrap()).await;
    assert!(matches!(closed, BankHttpMutationOutcome::Closed { .. }));
    assert_eq!(balance(&client, &base, "advanced").await, before + 17);
    let refused: BankHttpMutationOutcome = post(&client, &base, "/v1/mutations", &serde_json::json!({
        "protocol":"v4", "request_id":"stale-selection", "credential":credential(), "command":"create",
        "custody_id":"stale-selection", "sequence":1, "controls":{"deadline_milliseconds":5000,"maximum_history_work":128},
        "idempotency_key":"stale-selection", "operation":{"operation":"reapply_send_money","source":source,"history_token":completed_token}
    })).await;
    assert!(
        matches!(&refused, BankHttpMutationOutcome::Rejected { denial, .. }
        if *denial == BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)),
        "same-branch page: {refused:?}"
    );
    let mut resume = history_request("stale-continuation", &source, 1);
    resume["continuation"] = serde_json::json!(partial_token);
    let refused: BankHttpCorrectionHistoryOutcome =
        post(&client, &base, "/v1/corrections/history/resume", &resume).await;
    assert!(
        matches!(&refused, BankHttpCorrectionHistoryOutcome::Denied { denial, .. }
        if *denial == BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)),
        "same-branch continuation: {refused:?}"
    );
    assert_eq!(
        balance(&client, &base, "refusals").await,
        before + 17,
        "refused selectors cannot execute the retained transfer"
    );
    assert_fresh_pagination(&client, &base, &application, &principal, &source).await;

    // Native history stays available through the actual completed owners above.
    // Closing every owner releases those World leases; scalar chooser tokens
    // and Relational page boundaries cannot prolong them across publication.
    super::fixture::close_owners(&application, &principal, owners);
    let deposited = continuation.execute(serde_json::from_value(serde_json::json!({
        "protocol":"v4", "request_id":"retire", "credential":credential(), "command":"create",
        "custody_id":"retire", "sequence":1, "controls":{"deadline_milliseconds":5000,"maximum_history_work":128},
        "idempotency_key":"retire-source-history", "operation":{"operation":"deposit","institution":"fixture:1","account":"fixture:100","amount_minor_units":1}
    })).unwrap()).await;
    assert!(
        matches!(&deposited, BankHttpMutationOutcome::Completed { .. }),
        "retirement publication: {deposited:?}"
    );
    let closed = continuation.execute(serde_json::from_value(serde_json::json!({
        "protocol":"v4", "request_id":"close-retire", "credential":credential(), "command":"close",
        "custody_id":"retire", "sequence":2, "controls":{"deadline_milliseconds":5000}
    })).unwrap()).await;
    assert!(matches!(closed, BankHttpMutationOutcome::Closed { .. }));
    let retired_scope = scope();
    let cause = application
        .runtime
        .request(&principal, &retired_scope)
        .corrections()
        .select_coordinates(&source.clone().into(), limits())
        .err()
        .expect("the unprotected source's World ancestry retires");
    assert!(
        matches!(
            &cause,
            WorthQueryApplicationCorrectionDenial::Source(
                WorthQueryCorrectionSelectionDenial::Ancestry(RuntimeWorldServiceDenial::Denied(
                    ProductBranchAncestorSelectionDenial::History(
                        CompositeAncestorSelectionDenial::SourceUnavailable
                    )
                ))
            )
        ),
        "released native source ancestry: {cause:?}"
    );
    assert_eq!(
        correction_denial(cause),
        BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
    );
    let retired: BankHttpCorrectionHistoryOutcome = post(
        &client,
        &base,
        "/v1/corrections/history",
        &history_request("released-source", &source, 1),
    )
    .await;
    assert!(
        matches!(&retired, BankHttpCorrectionHistoryOutcome::Denied { denial, .. }
        if *denial == BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)),
        "released source history: {retired:?}; native: {}",
        native_initial_diagnostic(&application, &principal, &source)
    );
    assert_eq!(
        balance(&client, &base, "retirement-no-transfer").await,
        before + 18
    );

    let foreign = fixture().await;
    let foreign_scope = scope();
    let foreign_request = foreign
        .application
        .runtime
        .request(&foreign.principal, &foreign_scope);
    let selected = foreign_request
        .corrections()
        .select_coordinates(&foreign.source.clone().into(), limits())
        .unwrap();
    let foreign_page = foreign_request
        .corrections()
        .continue_alternatives(
            &selected,
            limits(),
            std::num::NonZeroUsize::MIN,
            foreign.page.continuation().unwrap(),
        )
        .unwrap()
        .into_page();
    let fresh_scope = scope();
    let cause = application
        .runtime
        .request(&principal, &fresh_scope)
        .corrections()
        .select_alternative_coordinates(&foreign_page, &source.into(), limits())
        .err()
        .expect("a foreign runtime page cannot select");
    assert!(
        matches!(
            &cause,
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::ForeignAlternative
            )
        ),
        "foreign owner: {cause:?}"
    );
    assert_eq!(
        correction_denial(cause),
        BankHttpDenial::new(
            BankHttpDenialKind::PermissionDenied,
            BankHttpNextAction::None
        )
    );
    server.shutdown().await.unwrap();
    assert!(continuation.finish().await.is_ok());
}

async fn assert_fresh_pagination(
    client: &reqwest::Client,
    base: &str,
    application: &crate::http::server::tests::fixture::CausalHttpApplication,
    principal: &bank_server::BankAuthenticatedPrincipal,
    source: &BankHttpCorrectionSource,
) {
    let initial = history_request("history-two-pages", source, 1);
    let mut page: BankHttpCorrectionHistoryOutcome =
        post(client, base, "/v1/corrections/history", &initial).await;
    assert!(
        matches!(&page, BankHttpCorrectionHistoryOutcome::Delivered { .. }),
        "fresh initial after rejected stale create: {page:?}; native: {}",
        native_initial_diagnostic(application, principal, source)
    );
    assert_eq!(
        post::<BankHttpCorrectionHistoryOutcome>(client, base, "/v1/corrections/history", &initial)
            .await,
        page,
        "fresh initial response replay after branch advance"
    );
    let mut observed = Vec::new();
    let mut offset = 0;
    let mut expected_id = "history-two-pages".to_owned();
    loop {
        let BankHttpCorrectionHistoryOutcome::Delivered {
            request_id,
            alternatives,
            continuation,
            complete,
        } = &page
        else {
            panic!("fresh continuation {expected_id} after branch advance: {page:?}");
        };
        assert_eq!(request_id, &expected_id);
        assert_eq!(
            alternatives.len(),
            1,
            "one canonical alternative per funded page"
        );
        observed.extend(alternatives.iter().map(|row| row.source.clone()));
        if *complete {
            break;
        }
        offset += 1;
        expected_id = format!("history-two-pages-{offset}");
        let mut request = history_request(&expected_id, source, 1);
        request["continuation"] = serde_json::json!(continuation);
        page = post(client, base, "/v1/corrections/history/resume", &request).await;
        assert_eq!(
            post::<BankHttpCorrectionHistoryOutcome>(
                client,
                base,
                "/v1/corrections/history/resume",
                &request
            )
            .await,
            page,
            "fresh continuation response replay for {expected_id}"
        );
    }
    assert_eq!(
        observed.len(),
        2,
        "the two native reversals survive the unrelated edit"
    );
    assert!(
        observed.contains(source),
        "the older reversal remains explicitly selectable"
    );
}

fn native_initial_diagnostic(
    application: &crate::http::server::tests::fixture::CausalHttpApplication,
    principal: &bank_server::BankAuthenticatedPrincipal,
    source: &BankHttpCorrectionSource,
) -> String {
    let request_scope = scope();
    let request = application.runtime.request(principal, &request_scope);
    let selected = match request
        .corrections()
        .select_coordinates(&source.clone().into(), limits())
    {
        Ok(selected) => selected,
        Err(cause) => return format!("source selection: {cause:?}"),
    };
    match request
        .corrections()
        .alternatives(&selected, limits(), std::num::NonZeroUsize::MIN)
    {
        Ok(page) => format!(
            "initial delivered {} rows, continuation {}",
            page.entries().len(),
            page.continuation().is_some()
        ),
        Err(cause) => format!("initial traversal: {cause:?}"),
    }
}

fn credential() -> serde_json::Value {
    serde_json::json!({"id_token":"test-only","access_token":"test-only","nonce":"test-only"})
}
fn history_request(id: &str, source: &BankHttpCorrectionSource, width: usize) -> serde_json::Value {
    serde_json::json!({"protocol":"v4","request_id":id,"credential":credential(),"source":source,
        "controls":{"deadline_milliseconds":5000,"maximum_history_work":128,"maximum_disclosure_results":16,"maximum_entries":width}})
}
async fn post<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    request: &serde_json::Value,
) -> T {
    client
        .post(format!("{base}{path}"))
        .json(request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
async fn balance(client: &reqwest::Client, base: &str, id: &str) -> i64 {
    let outcome: BankHttpAccountSummaryOutcome = post(
        client,
        base,
        "/v1/queries/account-summary",
        &serde_json::json!({
            "protocol":"v4","request_id":id,"credential":credential(),"account":"fixture:100",
            "controls":{"deadline_milliseconds":5000,"maximum_results":16}
        }),
    )
    .await;
    match outcome {
        BankHttpAccountSummaryOutcome::Delivered { summary, .. } => summary.current_balance_minor,
        other => panic!("independent account balance: {other:?}"),
    }
}
