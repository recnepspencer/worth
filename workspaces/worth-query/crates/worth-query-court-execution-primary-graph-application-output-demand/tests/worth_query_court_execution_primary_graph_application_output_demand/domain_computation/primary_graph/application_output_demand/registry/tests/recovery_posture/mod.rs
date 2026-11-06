pub use std::sync::Arc;
pub use std::sync::Condvar;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRecord;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandWake;
pub use std::sync::Mutex;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::demand_key;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key_with_identity;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::required_interest;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::root;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture;
#[test]
fn retryable_publication_stale_does_not_keep_a_closed_required_interest() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("required", 6, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
    .insert_for_test(demand_key.clone(), {
        let __required_interests = 1;
        let __wake = Arc::clone(&wake);
        let mut __base = record(occurrence, DemandState::Running, 1);
        *__base.required_interests_mut_for_test() = __required_interests;
        *__base.wake_mut_for_test() = __wake;
        __base
    });
    let demand_interest = required_interest(&registry, demand_key.clone(), wake);
    let mut denial = WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::PublicationStale,
        "a sibling advanced the product head",
    );

    registry.finish_execution_failure_for_test(&demand_interest, &mut denial);
    assert_eq!(
        denial.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Retryable
    );
    assert!(matches!(
        *(*(*registry.state_for_test())
            .lock()
            .unwrap()
            .records_for_test())[&demand_key]
            .state_for_test(),
        DemandState::Scheduled
    ));
    drop(demand_interest);

    assert!(!(*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_for_test())
    .contains_key_for_test(&demand_key));
}
#[test]
fn running_budget_failure_stays_with_its_request_and_leaves_the_row_claimable() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("required", 7, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
    .insert_for_test(demand_key.clone(), {
        let __required_interests = 1;
        let __wake = Arc::clone(&wake);
        let mut __base = record(occurrence(), DemandState::Running, 1);
        *__base.required_interests_mut_for_test() = __required_interests;
        *__base.wake_mut_for_test() = __wake;
        __base
    });
    let demand_interest = required_interest(&registry, demand_key.clone(), wake);
    let mut denial = WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "the running attempt exhausted its governed work budget",
    );

    registry.finish_execution_failure_for_test(&demand_interest, &mut denial);

    assert_eq!(
        denial.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Terminal
    );
    assert!(matches!(
        registry.begin_for_test(&demand_interest),
        WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of: None }
    ));
}
#[test]
fn no_change_checkpoint_keeps_its_real_receipt_across_claims() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let original_commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let original_occurrence = receipt.product_branch().occurrence_for_test();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("no-change", 8, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).insert_for_test(
        demand_key.clone(),
        { let __required_interests = 1; let __wake = Arc::clone(&wake); let mut __base = record(
                original_occurrence,
                DemandState::Output(WorthQueryOutputProgress::new_for_test(
                    WorthQueryOutputCheckpoint::Published {
                        receipt,
                        delivery: WorthQueryPendingOutputDelivery::NoChange,
                        ready_backing: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking::for_test_for_test(),
                    },
                )),
                1,
            ); *__base.required_interests_mut_for_test() = __required_interests; *__base.wake_mut_for_test() = __wake; __base },
    );
    let demand_interest = required_interest(&registry, demand_key.clone(), wake);
    let WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint } =
        registry.begin_for_test(&demand_interest)
    else {
        panic!("the no-change checkpoint must remain runnable")
    };
    assert_eq!(
        checkpoint
            .receipt()
            .expect("ordinary checkpoint receipt")
            .committed_product_publication()
            .composite_commit(),
        &original_commit
    );
    assert!(matches!(
        registry.begin_for_test(&demand_interest),
        WorthQueryOutputDemandAdvanceAdmission::Pending,
    ));

    registry
        .finish_checkpoint_for_test(&demand_interest, claim, checkpoint, None)
        .unwrap();

    let WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint {
        claim: retried_claim,
        checkpoint: retried,
    } = registry.begin_for_test(&demand_interest)
    else {
        panic!("releasing an idle claim must preserve no-change publication")
    };
    assert_ne!(claim, retried_claim);
    assert_eq!(
        retried
            .receipt()
            .expect("ordinary retry receipt")
            .committed_product_publication()
            .composite_commit(),
        &original_commit
    );
}
#[test]
fn closed_occurrence_stops_a_claim_without_losing_published_identity() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let occurrence = receipt.product_branch().occurrence_for_test();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("required", 9, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).insert_for_test(
        demand_key.clone(),
        { let __required_interests = 1; let __wake = Arc::clone(&wake); let mut __base = record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new_for_test(
                    WorthQueryOutputCheckpoint::Published {
                        receipt,
                        delivery: WorthQueryPendingOutputDelivery::NoChange,
                        ready_backing: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking::for_test_for_test(),
                    },
                )),
                1,
            ); *__base.required_interests_mut_for_test() = __required_interests; *__base.wake_mut_for_test() = __wake; __base },
    );
    let demand_interest = required_interest(&registry, demand_key.clone(), wake);
    let WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint } =
        registry.begin_for_test(&demand_interest)
    else {
        panic!("published output must admit one claim")
    };

    registry.release_product_occurrence_for_test(occurrence);
    let cause = registry
        .finish_checkpoint_for_test(&demand_interest, claim, checkpoint, None)
        .expect_err("branch close must win over the claimed completion");
    assert_eq!(cause.kind(), WorthQueryOutputDemandDenialKind::Closed);
    let state = (*registry.state_for_test()).lock().unwrap();
    let DemandState::Output(output) = (*state.records_for_test())[&demand_key].state_for_test()
    else {
        panic!("published identity must survive the terminal denial while interest remains")
    };
    assert_eq!(
        (*output.published_commit_for_test())
            .as_ref()
            .expect("ordinary published commit"),
        &commit,
    );
    assert_eq!(
        (*output.checkpoint_for_test())
            .as_ref()
            .expect("stopped output retains its checkpoint")
            .receipt()
            .expect("ordinary stopped checkpoint receipt")
            .committed_product_publication()
            .composite_commit(),
        &commit,
    );
    assert!(matches!(
        output.advancement_for_test(),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputAdvancement::Stopped { .. }
    ));
    drop(state);
    assert!(matches!(
        registry.begin_for_test(&demand_interest),
        WorthQueryOutputDemandAdvanceAdmission::Failed(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Closed
    ));
}

#[path = "checkpoint_capture/mod.rs"]
pub mod checkpoint_capture;
#[path = "restored_admission/mod.rs"]
pub mod restored_admission;
#[path = "successor/mod.rs"]
pub mod successor;
