use super::super::WorthQueryOutputProgress;
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandAdvanceAdmission;
use crate::domain_computation::primary_graph::application_output_demand::{
    WorthQueryOutputCheckpoint, WorthQueryPendingOutputDelivery,
};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture;

#[test]
fn retryable_publication_stale_keeps_required_scheduled_obligation() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("required", 6, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            required: true,
            wake: Arc::clone(&wake),
            ..record(occurrence, DemandState::Running, 1)
        },
    );
    let demand_interest = interest(&registry, demand_key.clone(), wake);
    let mut denial = WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::PublicationStale,
        "a sibling advanced the product head",
    );

    registry.finish_execution_failure(&demand_interest, &mut denial);
    assert_eq!(
        denial.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Retryable
    );
    assert!(matches!(
        registry.state.lock().unwrap().records[&demand_key].state,
        DemandState::Scheduled
    ));
    drop(demand_interest);

    let state = registry.state.lock().unwrap();
    let retained = &state.records[&demand_key];
    assert_eq!(retained.interests, 0);
    assert!(retained.required);
    assert!(matches!(retained.state, DemandState::Scheduled));
}

#[test]
fn running_budget_failure_cannot_claim_retry_after_the_registry_finishes_it() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("required", 7, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            required: true,
            wake: Arc::clone(&wake),
            ..record(occurrence(), DemandState::Running, 1)
        },
    );
    let demand_interest = interest(&registry, demand_key.clone(), wake);
    let mut denial = WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "the running attempt exhausted its governed work budget",
    );

    registry.finish_execution_failure(&demand_interest, &mut denial);

    assert_eq!(
        denial.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Terminal
    );
    assert!(matches!(
        registry.begin(&demand_interest),
        WorthQueryOutputDemandAdvanceAdmission::Failed(stored)
            if stored == denial
    ));
    assert_eq!(
        registry
            .reopen_stale_ready(&demand_interest, &receipt)
            .expect_err("a failed demand retains its terminal denial"),
        denial
    );
}

#[test]
fn stale_ready_receipt_reopens_the_same_demand_for_execution() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let unrelated = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = receipt.product_branch().occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("stale-ready", 8, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            required: true,
            wake: Arc::clone(&wake),
            ..record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new(
                    WorthQueryOutputCheckpoint::Ready(super::super::WorthQueryCompletedOutputDemand {
                        receipt: receipt.clone(),
                        readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
                    }),
                )),
                1,
            )
        },
    );
    let demand_interest = interest(&registry, demand_key.clone(), Arc::clone(&wake));

    assert!(matches!(
        registry.begin(&demand_interest),
        WorthQueryOutputDemandAdvanceAdmission::Ready(_)
    ));
    assert!(!registry
        .reopen_stale_ready(&demand_interest, &unrelated)
        .expect("an unrelated receipt cannot disturb the ready checkpoint"));
    assert!(matches!(
        registry.begin(&demand_interest),
        WorthQueryOutputDemandAdvanceAdmission::Ready(_)
    ));
    assert_eq!(*wake.generation.lock().unwrap(), 0);
    assert!(registry
        .reopen_stale_ready(&demand_interest, &receipt)
        .expect("the exact stale ready receipt can reopen"));
    assert_eq!(*wake.generation.lock().unwrap(), 1);
    let stale_key = *receipt.idempotency_binding().key_identity();
    assert!(matches!(
        registry.begin(&demand_interest),
        WorthQueryOutputDemandAdvanceAdmission::Execute {
            successor_of: Some(key)
        } if key == stale_key
    ));
    assert!(!registry
        .reopen_stale_ready(&demand_interest, &receipt)
        .expect("a running replacement cannot be reopened a second time"));
}

#[test]
fn no_change_checkpoint_keeps_its_real_receipt_across_claims() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let original_commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let original_occurrence = receipt.product_branch().occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("no-change", 8, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            required: true,
            wake: Arc::clone(&wake),
            ..record(
                original_occurrence,
                DemandState::Output(WorthQueryOutputProgress::new(
                    WorthQueryOutputCheckpoint::Published {
                        receipt,
                        delivery: WorthQueryPendingOutputDelivery::NoChange,
                    },
                )),
                1,
            )
        },
    );
    let demand_interest = interest(&registry, demand_key.clone(), wake);
    let WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint } =
        registry.begin(&demand_interest)
    else {
        panic!("the no-change checkpoint must remain runnable")
    };
    assert_eq!(
        checkpoint
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        &original_commit
    );
    assert!(matches!(
        registry.begin(&demand_interest),
        WorthQueryOutputDemandAdvanceAdmission::Pending,
    ));

    registry
        .finish_checkpoint(&demand_interest, claim, checkpoint, None)
        .unwrap();

    let WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint {
        claim: retried_claim,
        checkpoint: retried,
    } = registry.begin(&demand_interest)
    else {
        panic!("releasing an idle claim must preserve no-change publication")
    };
    assert_ne!(claim, retried_claim);
    assert_eq!(
        retried
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        &original_commit
    );
}

#[test]
fn closed_occurrence_stops_a_claim_without_losing_published_identity() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let stale_receipt = receipt.clone();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let occurrence = receipt.product_branch().occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("required", 9, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            required: true,
            wake: Arc::clone(&wake),
            ..record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new(
                    WorthQueryOutputCheckpoint::Published {
                        receipt,
                        delivery: WorthQueryPendingOutputDelivery::NoChange,
                    },
                )),
                1,
            )
        },
    );
    let demand_interest = interest(&registry, demand_key.clone(), wake);
    let WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint } =
        registry.begin(&demand_interest)
    else {
        panic!("published output must admit one claim")
    };

    registry.release_product_occurrence(occurrence);
    let cause = registry
        .finish_checkpoint(&demand_interest, claim, checkpoint, None)
        .expect_err("branch close must win over the claimed completion");
    assert_eq!(cause.kind(), WorthQueryOutputDemandDenialKind::Closed);
    let state = registry.state.lock().unwrap();
    let DemandState::Output(output) = &state.records[&demand_key].state else {
        panic!("published identity must survive the terminal denial while interest remains")
    };
    assert_eq!(
        output
            .receipt
            .committed_product_publication()
            .composite_commit(),
        &commit,
    );
    assert_eq!(
        output
            .checkpoint
            .as_ref()
            .expect("stopped output retains its checkpoint")
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        &commit,
    );
    assert!(matches!(
        output.advancement,
        super::super::WorthQueryOutputAdvancement::Stopped { .. }
    ));
    drop(state);
    assert_eq!(
        registry
            .reopen_stale_ready(&demand_interest, &stale_receipt)
            .expect_err("a stopped checkpoint retains its terminal denial")
            .kind(),
        WorthQueryOutputDemandDenialKind::Closed
    );
}
