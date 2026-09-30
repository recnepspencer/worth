//! A late rail callback cannot reopen a cancelled, callback-settled payment.

use std::time::Duration;

use bank_domain::schema::ApprovePayment;
use bank_external_rail::completion_wire::{CUSTODY_ACK_V1_BYTES, CUSTODY_ACK_V1_MAGIC};
use bank_external_rail::{inquire_completed_effect_count, inquire_completion_delivery_posture};
use bank_server::BankApprovedPaymentWorkflowError;
use worth_query_host::facade::application_entry::{
    WorkflowInstanceCancellationOutcome, WorkflowTransitionPreparationDenial,
    WorthQueryWorkflowAdvancePreparationDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationAttemptDenialKind;

use super::assertions::{key, require_completed};
use super::fixture::{principal_id, APPROVER};
use super::journey;
use super::oracle;
use super::setup::PaymentCourt;
use super::{correlation_token, support, TIMEOUT};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn callback_settled_cancellation_survives_late_exact_duplicate() {
    let mut court = PaymentCourt::start("payment-callback-cancellation").await;
    let scope = support::request_scope();
    let authority = ApprovePayment {
        payment: court.payment_id,
        approver: principal_id(APPROVER),
    };
    let before = tokio::task::block_in_place(|| {
        oracle::snapshot(
            &court.runtime,
            &court.approver,
            &court.recipient,
            court.source_account,
            court.destination_account,
        )
    });
    let (instance, operation) = tokio::task::block_in_place(|| {
        let workflow = court
            .runtime
            .approved_business_payment(&court.approver, &scope);
        journey::prepare_approved_payment_operation(&workflow, &authority)
    });
    tokio::task::block_in_place(|| {
        court
            .runtime
            .approved_business_payment(&court.approver, &scope)
            .perform_apply(
                instance.clone(),
                &operation,
                authority.clone(),
                &key("approved-payment:cancel-court:perform"),
            )
            .expect("the real payment operation commits")
            .into_performed()
            .expect("the rail owns the committed payment effect");
    });

    let signed = tokio::time::timeout(Duration::from_secs(20), court.proxy.await_first_capture())
        .await
        .expect("rail sends a signed completion");
    let token = correlation_token(&signed);
    court.proxy.release_first();
    let terminal = tokio::time::timeout(Duration::from_secs(25), async {
        loop {
            if let Some(terminal) = court.bank.observe_rail_completion(token) {
                if inquire_completion_delivery_posture(court.rail.local_addr(), TIMEOUT)
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
    .expect("callback publishes one canonical World terminal");
    let after_payment = tokio::task::block_in_place(|| {
        oracle::snapshot(
            &court.runtime,
            &court.approver,
            &court.recipient,
            court.source_account,
            court.destination_account,
        )
    });
    oracle::assert_one_payment(&before, &after_payment);

    let workflow = court
        .runtime
        .approved_business_payment(&court.approver, &scope);
    require_completed(
        tokio::task::block_in_place(|| {
            workflow.accept_applied(
                instance.clone(),
                &operation,
                authority.clone(),
                authority.clone(),
                &key("approved-payment:cancel-court:perform"),
                &key("approved-payment:cancel-court:accept"),
            )
        })
        .expect("fresh owner acceptance sees canonical inbound completion"),
        "apply",
    );
    let cancelled = tokio::task::block_in_place(|| {
        workflow
            .prepare_cancellation(
                instance.clone(),
                authority.clone(),
                &key("approved-payment:cancel-court:cancel"),
            )
            .expect("settled operation releases owner custody")
            .execute()
    });
    match cancelled {
        WorkflowInstanceCancellationOutcome::Cancelled(done) => {
            assert_eq!(done.performed_node_paths(), ["apply".to_owned()]);
        }
        other => panic!("callback-settled instance must cancel: {other:?}"),
    }

    let duplicate = reqwest::Client::new()
        .post(format!(
            "http://{}/v1/inbound/rail-completions",
            court.bank.local_address()
        ))
        .body(signed)
        .send()
        .await
        .expect("late exact callback reaches Bank HTTP");
    assert!(duplicate.status().is_success());
    let ack = duplicate
        .bytes()
        .await
        .expect("signed duplicate ACK is readable");
    assert_eq!(ack.len(), CUSTODY_ACK_V1_BYTES);
    assert_eq!(&ack[..16], CUSTODY_ACK_V1_MAGIC);
    assert_eq!(
        ack[16], 2,
        "the exact signed message replays its performed custody"
    );
    let retained = court
        .bank
        .observe_rail_completion(token)
        .expect("duplicate retains the canonical terminal");
    assert_eq!(
        retained.original_world_commit(),
        terminal.original_world_commit()
    );
    assert_eq!(
        retained.completion_world_commit(),
        terminal.completion_world_commit()
    );
    let denied = tokio::task::block_in_place(|| {
        workflow.advance(
            instance.clone(),
            authority.clone(),
            &key("approved-payment:cancel-court:late-advance"),
        )
    });
    match denied {
        Err(BankApprovedPaymentWorkflowError::Advance(
            WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
                WorkflowTransitionPreparationDenial::Attempt(attempt),
            ),
        )) => assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled,
            "duplicate callback cannot reopen a cancelled instance"
        ),
        other => panic!("cancelled instance must name its terminal denial: {other:?}"),
    }
    let after_duplicate = tokio::task::block_in_place(|| {
        oracle::snapshot(
            &court.runtime,
            &court.approver,
            &court.recipient,
            court.source_account,
            court.destination_account,
        )
    });
    assert_eq!(after_duplicate, after_payment);
    assert_eq!(
        inquire_completed_effect_count(court.rail.local_addr(), TIMEOUT)
            .await
            .expect("rail reports one real consequence"),
        1
    );
    tokio::task::block_in_place(|| assert_eq!(court.transport.admission_count(), 1));
    drop(workflow);
    court.proxy.shutdown().await;
    court.bank.shutdown().await.expect("callback host stops");
    tokio::task::block_in_place(|| {
        drop(court.runtime);
        drop(court.transport);
        drop(court.rail);
    });
}
