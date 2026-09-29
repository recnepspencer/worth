//! Bank payment workflow, separate rail consequence, and real callback HTTP.
//!
//! The approval principals enter through Bank's causal Query authentication
//! adapter. The callback crosses a separate OS process and the production HTTP
//! verifier; this court does not claim an OIDC payment entry route.

use std::time::{Duration, Instant};

use bank_domain::queries::payment;
use bank_domain::schema::{ApprovePayment, PaymentStatus};
use bank_external_rail::{inquire_completed_effect_count, inquire_completion_delivery_posture};
use bank_server::BankApprovedPaymentWorkflowError;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind;

use crate::support;

#[path = "payment/authority.rs"]
mod authority;
#[path = "payment/cancellation.rs"]
mod cancellation;
#[path = "payment/close.rs"]
mod close;
#[path = "payment/oracle.rs"]
mod oracle;
#[path = "payment/retirement.rs"]
mod retirement;
#[path = "payment/setup.rs"]
mod setup;

#[path = "../../../bank-server/tests/approved_payment_workflow/approval.rs"]
mod approval;
#[path = "../../../bank-server/tests/approved_payment_workflow/assertions.rs"]
mod assertions;
#[path = "../../../bank-server/tests/approved_payment_workflow/authentication.rs"]
mod authentication;
#[allow(dead_code, reason = "shared Bank fixture")]
#[path = "../../../bank-server/tests/ordinary_reads/fixture.rs"]
mod fixture;
#[path = "../../../bank-server/tests/approved_payment_workflow/journey.rs"]
mod journey;
#[allow(dead_code, reason = "shared rail adapter exposes broader owner probes")]
#[path = "../../../bank-server/tests/ordinary_mutations/estate_operations/external_effect_dispatch/rail_transport.rs"]
mod rail_transport;

use super::rail_completion::proxy::{correlation_token, CallbackProxy};
use assertions::{key, require_completed};
use fixture::{principal_id, APPROVER};
use setup::PaymentCourt;

const TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn completed_payment_callback_waits_for_fresh_workflow_advance() {
    let PaymentCourt {
        rail,
        runtime,
        transport,
        bank,
        mut proxy,
        approver,
        owner,
        recipient,
        payment_id,
        source_account,
        destination_account,
    } = PaymentCourt::start("payment-inbound-process").await;

    let request_cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        request_cancellation.token(),
    );
    let authority = ApprovePayment {
        payment: payment_id,
        approver: principal_id(APPROVER),
    };
    let (instance, operation) = tokio::task::block_in_place(|| {
        let workflow = runtime.approved_business_payment(&approver, &scope);
        journey::prepare_approved_payment_operation(&workflow, &authority)
    });
    assert_eq!(instance.branch(), runtime.current_branch());
    let before_accounts = tokio::task::block_in_place(|| {
        oracle::snapshot(
            &runtime,
            &approver,
            &recipient,
            source_account,
            destination_account,
        )
    });
    let initial = tokio::task::block_in_place(|| {
        let workflow = runtime.approved_business_payment(&approver, &scope);
        workflow
            .perform_apply(
                instance.clone(),
                &operation,
                authority.clone(),
                &key("approved-payment:operation:perform"),
            )
            .expect("real payment operation commits before response loss")
    });
    assert!(initial
        .into_performed()
        .expect("payment commit performed")
        .newly_committed());

    let first = tokio::time::timeout(Duration::from_secs(20), proxy.await_first_capture())
        .await
        .expect("the rail sends its completed payment callback");
    let token = correlation_token(&first);
    assert!(bank.observe_rail_completion(token).is_none());
    assert_eq!(
        inquire_completion_delivery_posture(rail.local_addr(), TIMEOUT)
            .await
            .expect("rail retains held callback")
            .pending,
        1
    );
    let pending = tokio::task::block_in_place(|| {
        runtime
            .approved_business_payment(&approver, &scope)
            .advance(
                instance.clone(),
                authority.clone(),
                &key("approved-payment:advance:held-callback"),
            )
            .expect("held callback cannot advance the workflow")
    });
    assert!(matches!(
        pending,
        WorkflowProgressOutcome::AwaitingOperation(_)
    ));
    request_cancellation.cancel();
    drop(scope);
    let owner_scope = support::request_scope();
    tokio::task::block_in_place(|| {
        authority::revoke_approver(
            &runtime,
            &owner,
            &owner_scope,
            instance.branch(),
            source_account,
            principal_id(APPROVER),
        )
    });
    proxy.release_first();

    let terminal = tokio::time::timeout(Duration::from_secs(25), async {
        loop {
            if let Some(terminal) = bank.observe_rail_completion(token) {
                if inquire_completion_delivery_posture(rail.local_addr(), TIMEOUT)
                    .await
                    .expect("rail reports callback custody")
                    .pending
                    == 0
                {
                    break terminal;
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("lost ACK retries to one World-performed payment terminal");
    assert_ne!(
        terminal.original_world_commit(),
        terminal.completion_world_commit()
    );
    assert_eq!(
        proxy.captured().len(),
        2,
        "one lost ACK causes one exact retry"
    );
    assert_eq!(
        inquire_completed_effect_count(rail.local_addr(), TIMEOUT)
            .await
            .expect("rail reports actual payment consequence"),
        1
    );
    runtime.release_workflow_instance_progress_for_test();

    let revoked_scope = support::request_scope();
    let revoked = tokio::task::block_in_place(|| {
        runtime
            .approved_business_payment(&approver, &revoked_scope)
            .advance(
                instance.clone(),
                authority.clone(),
                &key("approved-payment:advance:revoked-after-callback"),
            )
    });
    assert!(matches!(
        revoked,
        Err(BankApprovedPaymentWorkflowError::Advance(
            WorthQueryWorkflowAdvancePreparationDenial::AwaitingActor(ref required)
        )) if required.instance() == instance.entity_id()
            && required.denial().kind()
                == WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
    ));
    tokio::task::block_in_place(|| {
        authority::restore_approver(
            &runtime,
            &owner,
            &owner_scope,
            source_account,
            principal_id(APPROVER),
        )
    });
    let scope = support::request_scope();
    let still_pending = tokio::task::block_in_place(|| {
        runtime
            .approved_business_payment(&approver, &scope)
            .advance(
                instance.clone(),
                authority.clone(),
                &key("approved-payment:advance:terminal-before-owner"),
            )
            .expect("World terminal still requires explicit owner acceptance")
    });
    assert!(matches!(
        still_pending,
        WorkflowProgressOutcome::AwaitingOperation(_)
    ));
    let workflow = runtime.approved_business_payment(&approver, &scope);
    require_completed(
        tokio::task::block_in_place(|| {
            workflow.accept_applied(
                instance.clone(),
                &operation,
                authority.clone(),
                authority.clone(),
                &key("approved-payment:operation:perform"),
                &key("approved-payment:operation:accept:inbound"),
            )
        })
        .expect("fresh authorized owner acceptance settles the exact operation"),
        "apply",
    );
    require_completed(
        tokio::task::block_in_place(|| {
            workflow.advance(
                instance.clone(),
                authority.clone(),
                &key("approved-payment:advance:await-inbound"),
            )
        })
        .expect("fresh advance consumes the typed inbound wait"),
        "await-inbound",
    );
    require_completed(
        tokio::task::block_in_place(|| {
            workflow.advance(
                instance.clone(),
                authority.clone(),
                &key("approved-payment:advance:terminal"),
            )
        })
        .expect("one successor becomes reachable"),
        "completed",
    );

    tokio::task::block_in_place(|| {
        let payment = runtime
            .request(&approver, &scope)
            .on_branch(instance.branch())
            .query(payment(payment_id))
            .execute()
            .expect("Bank payment readback succeeds");
        assert_eq!(payment.rows()[0].status(), PaymentStatus::Committed);
    });
    let after_accounts = tokio::task::block_in_place(|| {
        oracle::snapshot(
            &runtime,
            &approver,
            &recipient,
            source_account,
            destination_account,
        )
    });
    oracle::assert_one_payment(&before_accounts, &after_accounts);
    tokio::task::block_in_place(|| {
        assert_eq!(transport.admission_count(), 1);
        assert_eq!(transport.completed_effect_count(), 1);
    });
    tokio::task::block_in_place(|| {
        retirement::adopt_p2_after_completed_payment(
            &runtime,
            &approver,
            &scope,
            instance.branch(),
            authority,
        )
    });
    let retained = bank
        .observe_rail_completion(token)
        .expect("program retirement retains the original World terminal");
    assert_eq!(
        retained.original_world_commit(),
        terminal.original_world_commit()
    );
    assert_eq!(
        retained.completion_world_commit(),
        terminal.completion_world_commit()
    );
    let after_adoption = tokio::task::block_in_place(|| {
        oracle::snapshot(
            &runtime,
            &approver,
            &recipient,
            source_account,
            destination_account,
        )
    });
    assert_eq!(after_adoption, after_accounts);
    proxy.shutdown().await;
    bank.shutdown().await.expect("callback host stops");
    tokio::task::block_in_place(|| {
        drop(runtime);
        drop(transport);
        drop(rail);
    });
}
