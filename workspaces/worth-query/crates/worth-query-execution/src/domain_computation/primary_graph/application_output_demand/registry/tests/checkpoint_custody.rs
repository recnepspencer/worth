//! Registry claim mechanics over real committed receipts and prepaid cells.
//! Row replacement and stop schedules are assigned directly; this is not a
//! producer execution or World publication fixture.
use super::super::{
    CheckpointFinishStop, PreparedReadyBacking, WorthQueryOutputCheckpoint as Checkpoint,
    WorthQueryOutputDemandAdvanceAdmission as Admission, WorthQueryOutputProgress,
    WorthQueryPendingOutputDelivery,
};
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

fn published(receipt: WorthQueryApplicationCommitReceipt) -> Checkpoint {
    Checkpoint::Published {
        receipt,
        delivery: WorthQueryPendingOutputDelivery::NoChange,
        ready_backing: PreparedReadyBacking::for_test(),
    }
}
fn insert(
    registry: &WorthQueryOutputDemandRegistry,
    key: &WorthQueryOutputDemandKey,
    checkpoint: Checkpoint,
) {
    let occurrence = checkpoint.receipt().unwrap().product_branch().occurrence();
    registry.state.lock().unwrap().records.insert(
        key.clone(),
        record(
            occurrence,
            DemandState::Output(WorthQueryOutputProgress::new(checkpoint)),
            1,
        ),
    );
}
fn claim(
    registry: &WorthQueryOutputDemandRegistry,
    owner: &WorthQueryOutputDemandInterest,
) -> (super::super::WorthQueryOutputClaimIdentity, Checkpoint) {
    match registry.begin(owner) {
        Admission::AdvanceCheckpoint { claim, checkpoint } => (claim, checkpoint),
        _ => panic!("real checkpoint is claimable"),
    }
}
fn owner(
    registry: &WorthQueryOutputDemandRegistry,
    key: WorthQueryOutputDemandKey,
) -> WorthQueryOutputDemandInterest {
    let wake = Arc::clone(&registry.state.lock().unwrap().records[&key].wake);
    interest(registry, key, wake)
}
fn returned(
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
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let registry = WorthQueryOutputDemandRegistry::default();
    let key = key("same-claim-new-cell", 1, 1);
    insert(&registry, &key, published(receipt.clone()));
    let owner = owner(&registry, key.clone());
    let (old_claim, old_checkpoint) = claim(&registry, &owner);
    let Checkpoint::Published { ready_backing, .. } = &old_checkpoint else {
        unreachable!()
    };
    let original = ready_backing.generation();

    insert(&registry, &key, published(receipt));
    let (new_claim, new_checkpoint) = claim(&registry, &owner);
    assert_eq!(old_claim, new_claim, "the ordinal collision is intentional");
    let old_checkpoint = returned(
        registry.finish_checkpoint_owned(&owner, old_claim, old_checkpoint, None),
        WorthQueryOutputDemandDenialKind::SchedulingRejected,
    );
    let Checkpoint::Published { ready_backing, .. } = &old_checkpoint else {
        unreachable!()
    };
    assert!(
        original.same(&ready_backing.generation()),
        "actual old cell returned"
    );
    assert!(
        registry
            .finish_checkpoint_owned(&owner, new_claim, new_checkpoint, None)
            .is_ok(),
        "the genuine replacement cell is lawful under the equal ordinal"
    );
    let state = registry.state.lock().unwrap();
    let DemandState::Output(output) = &state.records[&key].state else {
        unreachable!()
    };
    assert!(!output.owns_checkpoint(&old_checkpoint));
    assert!(output
        .checkpoint
        .as_ref()
        .is_some_and(|checkpoint| output.owns_checkpoint(checkpoint)));
}

#[test]
fn foreign_registry_and_missing_record_return_the_exact_claimed_cell() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let registry = WorthQueryOutputDemandRegistry::default();
    let key = key("foreign-registry", 1, 2);
    insert(&registry, &key, published(receipt));
    let owner = owner(&registry, key.clone());
    let (claim, checkpoint) = claim(&registry, &owner);
    let Checkpoint::Published { ready_backing, .. } = &checkpoint else {
        unreachable!()
    };
    let original = ready_backing.generation();
    let foreign = WorthQueryOutputDemandRegistry::default();
    let checkpoint = returned(
        foreign.finish_checkpoint_owned(&owner, claim, checkpoint, None),
        WorthQueryOutputDemandDenialKind::ForeignDemand,
    );
    let removed = registry.state.lock().unwrap().records.remove(&key);
    drop(removed);
    let checkpoint = returned(
        registry.finish_checkpoint_owned(&owner, claim, checkpoint, None),
        WorthQueryOutputDemandDenialKind::Closed,
    );
    let Checkpoint::Published { ready_backing, .. } = &checkpoint else {
        unreachable!()
    };
    assert!(original.same(&ready_backing.generation()));
}

#[test]
fn cleanup_claim_keeps_the_original_stop_and_refuses_a_second_borrow() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = receipt.product_branch().occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let key = key("stopped-cleanup", 1, 3);
    insert(&registry, &key, published(receipt));
    let owner = owner(&registry, key.clone());
    let (first_claim, checkpoint) = claim(&registry, &owner);
    registry.release_product_occurrence(occurrence);
    let stop = match registry.finish_checkpoint_owned(&owner, first_claim, checkpoint, None) {
        Err(CheckpointFinishStop::Restored(stop)) => stop,
        _ => panic!("the original stopped claim is restored"),
    };
    assert_eq!(stop.kind(), WorthQueryOutputDemandDenialKind::Closed);
    let (cleanup_claim, checkpoint) = {
        let mut state = registry.state.lock().unwrap();
        let mut row = state.records.get_mut(&key).unwrap();
        let DemandState::Output(output) = &mut row.state else {
            unreachable!()
        };
        let claimed = output
            .claim_stopped_checkpoint_for_close()
            .expect("present custody can be disposed");
        assert_ne!(claimed.0, first_claim);
        assert!(
            output.claim_stopped_checkpoint_for_close().is_none(),
            "no double loan"
        );
        row.finish();
        claimed
    };
    assert!(matches!(registry.begin(&owner), Admission::Failed(ref denial) if denial == &stop));
    assert!(
        owner.require_read_closeable().is_err(),
        "a stopped outstanding loan retains custody"
    );
    {
        let state = registry.state.lock().unwrap();
        assert!(matches!(
            state.records[&key].idle_disposition(false),
            super::super::disposition::IdleRowDisposition::InFlight
        ));
    }
    assert!(
        matches!(registry.finish_checkpoint_owned(&owner, cleanup_claim, checkpoint, None),
        Err(CheckpointFinishStop::Restored(ref denial)) if denial == &stop)
    );
    assert!(matches!(registry.begin(&owner), Admission::Failed(ref denial) if denial == &stop));
}
