//! Orderly Bank close reports a real dispatch while its callback is held.

use bank_http_adapter::BankRailCloseAssessment;

use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn close_before_payment_callback_preserves_unresolved_dispatch() {
    let PaymentCourt {
        rail,
        runtime,
        transport,
        bank,
        mut proxy,
        approver,
        payment_id,
        ..
    } = PaymentCourt::start("payment-close-before-callback").await;
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let authority = ApprovePayment {
        payment: payment_id,
        approver: principal_id(APPROVER),
    };
    let (instance, operation) = tokio::task::block_in_place(|| {
        let workflow = runtime.approved_business_payment(&approver, &scope);
        journey::prepare_approved_payment_operation(&workflow, &authority)
    });
    tokio::task::block_in_place(|| {
        let workflow = runtime.approved_business_payment(&approver, &scope);
        assert!(workflow
            .perform_apply(
                instance,
                &operation,
                authority,
                &key("payment-close-before-callback:apply"),
            )
            .expect("payment dispatch commits")
            .into_performed()
            .expect("payment commit performed")
            .newly_committed());
    });
    let envelope = tokio::time::timeout(Duration::from_secs(20), proxy.await_first_capture())
        .await
        .expect("rail callback proves completed consequence");
    let token = correlation_token(&envelope);
    assert!(bank.observe_rail_completion(token).is_none());

    let close = bank.shutdown().await.expect("callback host closes orderly");
    assert!(matches!(
        close.assessment(),
        BankRailCloseAssessment::Assessed {
            outstanding_dispatches: 1..,
            retained_accepted_occurrences: 0,
            ..
        }
    ));
    let continuation = close.into_continuation();
    proxy.shutdown().await;
    let rail_close = rail.close().expect("rail reports pending sender custody");
    assert_eq!(rail_close.pending, 1);
    tokio::task::block_in_place(|| {
        drop(runtime);
        drop(transport);
        drop(continuation);
    });
}
