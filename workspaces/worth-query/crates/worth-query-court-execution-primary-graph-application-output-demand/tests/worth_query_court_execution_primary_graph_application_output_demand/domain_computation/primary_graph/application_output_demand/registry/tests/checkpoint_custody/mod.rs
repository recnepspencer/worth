//! Registry claim mechanics over real committed receipts and prepaid cells.
//! Row replacement and stop schedules are assigned directly; this is not a
//! producer execution or World publication fixture.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::progression::checkpoint_finish::CheckpointFinishStop;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint as Checkpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission as Admission;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;

pub fn published(receipt: WorthQueryApplicationCommitReceipt) -> Checkpoint {
    Checkpoint::Published {
        receipt,
        delivery: WorthQueryPendingOutputDelivery::NoChange,
        ready_backing: PreparedReadyBacking::for_test_for_test(),
    }
}
pub fn insert(
    registry: &WorthQueryOutputDemandRegistry,
    key: &WorthQueryOutputDemandKey,
    checkpoint: Checkpoint,
) {
    let occurrence = checkpoint.receipt().unwrap().product_branch().occurrence_for_test();
    (*(*registry.state_for_test()).lock().unwrap().records_for_test()).insert_for_test(
        key.clone(),
        record(
            occurrence,
            DemandState::Output(WorthQueryOutputProgress::new_for_test(checkpoint)),
            1,
        ),
    );
}
pub fn claim(
    registry: &WorthQueryOutputDemandRegistry,
    owner: &WorthQueryOutputDemandInterest,
) -> (worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputClaimIdentity, Checkpoint){
    match registry.begin_for_test(owner) {
        Admission::AdvanceCheckpoint { claim, checkpoint } => (claim, checkpoint),
        _ => panic!("real checkpoint is claimable"),
    }
}
pub fn owner(
    registry: &WorthQueryOutputDemandRegistry,
    key: WorthQueryOutputDemandKey,
) -> WorthQueryOutputDemandInterest {
    let wake = Arc::clone(&(*(*(*registry.state_for_test()).lock().unwrap().records_for_test())[&key].wake_for_test()));
    interest(registry, key, wake)
}
pub fn returned(
    result: Result<(), CheckpointFinishStop>,
    kind: WorthQueryOutputDemandDenialKind,
) -> Checkpoint {
    match result {
        Err(CheckpointFinishStop::Returned(stop, checkpoint)) => {
            assert_eq!(stop.kind(), kind);
            checkpoint
        }
        _ => panic!("the exact original checkpoint must return to its caller"),
    }
}
#[test]
fn old_cell_cannot_finish_a_replacement_row_with_the_same_numeric_claim() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let registry = WorthQueryOutputDemandRegistry::default();
    let key = key("same-claim-new-cell", 1, 1);
    insert(&registry, &key, published(receipt.clone()));
    let owner = owner(&registry, key.clone());
    let (old_claim, old_checkpoint) = claim(&registry, &owner);
    let Checkpoint::Published { ready_backing, .. } = &old_checkpoint else {
        unreachable!()
    };
    let original = ready_backing.generation_for_test();

    insert(&registry, &key, published(receipt));
    let (new_claim, new_checkpoint) = claim(&registry, &owner);
    assert_eq!(old_claim, new_claim, "the ordinal collision is intentional");
    let old_checkpoint = returned(
        registry.finish_checkpoint_owned_for_test(&owner, old_claim, old_checkpoint, None),
        WorthQueryOutputDemandDenialKind::SchedulingRejected,
    );
    let Checkpoint::Published { ready_backing, .. } = &old_checkpoint else {
        unreachable!()
    };
    assert!(
        original.same_for_test(&ready_backing.generation_for_test()),
        "actual old cell returned"
    );
    assert!(
        registry
            .finish_checkpoint_owned_for_test(&owner, new_claim, new_checkpoint, None)
            .is_ok(),
        "the genuine replacement cell is lawful under the equal ordinal"
    );
    let state = (*registry.state_for_test()).lock().unwrap();
    let DemandState::Output(output) = &(*(*state.records_for_test())[&key].state_for_test()) else {
        unreachable!()
    };
    assert!(!output.owns_checkpoint_for_test(&old_checkpoint));
    assert!((*output
        .checkpoint_for_test())
        .as_ref()
        .is_some_and(|checkpoint| output.owns_checkpoint_for_test(checkpoint)));
}

#[test]
fn foreign_registry_and_missing_record_return_the_exact_claimed_cell() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let registry = WorthQueryOutputDemandRegistry::default();
    let key = key("foreign-registry", 1, 2);
    insert(&registry, &key, published(receipt));
    let owner = owner(&registry, key.clone());
    let (claim, checkpoint) = claim(&registry, &owner);
    let Checkpoint::Published { ready_backing, .. } = &checkpoint else {
        unreachable!()
    };
    let original = ready_backing.generation_for_test();
    let foreign = WorthQueryOutputDemandRegistry::default();
    let checkpoint = returned(
        foreign.finish_checkpoint_owned_for_test(&owner, claim, checkpoint, None),
        WorthQueryOutputDemandDenialKind::ForeignDemand,
    );
    let removed = (*(*registry.state_for_test()).lock().unwrap().records_for_test()).remove_for_test(&key);
    drop(removed);
    let checkpoint = returned(
        registry.finish_checkpoint_owned_for_test(&owner, claim, checkpoint, None),
        WorthQueryOutputDemandDenialKind::Closed,
    );
    let Checkpoint::Published { ready_backing, .. } = &checkpoint else {
        unreachable!()
    };
    assert!(original.same_for_test(&ready_backing.generation_for_test()));
}

#[test]
fn cleanup_claim_keeps_the_original_stop_and_refuses_a_second_borrow() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = receipt.product_branch().occurrence_for_test();
    let registry = WorthQueryOutputDemandRegistry::default();
    let key = key("stopped-cleanup", 1, 3);
    insert(&registry, &key, published(receipt));
    let owner = owner(&registry, key.clone());
    let (first_claim, checkpoint) = claim(&registry, &owner);
    registry.release_product_occurrence_for_test(occurrence);
    let stop = match registry.finish_checkpoint_owned_for_test(&owner, first_claim, checkpoint, None) {
        Err(CheckpointFinishStop::Restored(stop)) => stop,
        _ => panic!("the original stopped claim is restored"),
    };
    assert_eq!(stop.kind(), WorthQueryOutputDemandDenialKind::Closed);
    let (cleanup_claim, checkpoint) = {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        let mut row = (*state.records_for_test()).get_mut(&key).unwrap();
        let DemandState::Output(output) = &mut (*row.state_mut_for_test()) else {
            unreachable!()
        };
        let claimed = output
            .claim_stopped_checkpoint_for_close_for_test()
            .expect("present custody can be disposed");
        assert_ne!(claimed.0, first_claim);
        assert!(
            output.claim_stopped_checkpoint_for_close_for_test().is_none(),
            "no double loan"
        );
        row.finish_for_test();
        claimed
    };
    assert!(matches!(registry.begin_for_test(&owner), Admission::Failed(ref denial) if denial == &stop));
    assert!(
        owner.require_read_closeable().is_err(),
        "a stopped outstanding loan retains custody"
    );
    {
        let state = (*registry.state_for_test()).lock().unwrap();
        assert!(matches!(
            (*state.records_for_test())[&key].idle_disposition_for_test(false),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::disposition::IdleRowDisposition::InFlight
        ));
    }
    assert!(
        matches!(registry.finish_checkpoint_owned_for_test(&owner, cleanup_claim, checkpoint, None),
        Err(CheckpointFinishStop::Restored(ref denial)) if denial == &stop)
    );
    assert!(matches!(registry.begin_for_test(&owner), Admission::Failed(ref denial) if denial == &stop));
}
