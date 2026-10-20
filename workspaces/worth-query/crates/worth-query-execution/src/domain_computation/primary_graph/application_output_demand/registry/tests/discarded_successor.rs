//! Refused frame custody must leave a delivery entry or a closed checkpoint.
use super::super::{
    WorthQueryOutputCheckpoint as Checkpoint, WorthQueryOutputDemandAdvanceAdmission as Admission,
};
use super::published_retirement::{admit, publish};
use super::*;

fn refuse_custody(registry: &WorthQueryOutputDemandRegistry) {
    let mut state = registry.state.lock().unwrap();
    // All existing storage remains funded. This admission has no room for
    // one more frame's actual, positive interest storage.
    state.required_budget_bytes = state.required_reserved_bytes
        + state
            .required_custody_retained_bytes
            .load(std::sync::atomic::Ordering::Acquire);
    drop(state);
    let stop = registry.reserve_held_successor_capacity(1).err().unwrap();
    assert_eq!(
        stop.kind(),
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
    );
}

#[test]
fn a_discarded_successor_with_only_a_claim_cannot_freeze_the_head() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    let source = WorthQueryOutputDemandInterest::source_for_typed_frame_test();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let old = key("claim-without-delivery", 1, 1);
        let old_owner = admit(&registry, &old);
        publish(&registry, &old_owner, receipt.clone());
        completes_once(&registry, &old_owner);
        // An actual dependent claim retains the occurrence, but owns no
        // executable entry with which to deliver the successor's checkpoint.
        registry
            .state
            .lock()
            .unwrap()
            .records
            .get_mut(&old)
            .unwrap()
            .framework_required_count = 1;
        drop(old_owner);
        let newest = key("claim-without-delivery", 2, 1);
        let owner = admit(&registry, &newest);
        publish(&registry, &owner, receipt);
        let frame = owner.real_typed_frame_for_registry_test(source);
        refuse_custody(&registry);
        frame();
        assert_eq!(registry.publication_counts_for_registry_test(), (0, 0));
        let state = registry.state.lock().unwrap();
        assert!(
            !state.records.contains_key(&newest),
            "the closed, unowned checkpoint is retired by the existing release"
        );
        assert_eq!(state.records[&old].framework_required_count, 1);
    });
}

fn completes_once(
    registry: &WorthQueryOutputDemandRegistry,
    owner: &WorthQueryOutputDemandInterest,
) {
    let Admission::AdvanceCheckpoint { claim, checkpoint } = registry.begin(owner) else {
        panic!("the entry takes its Published checkpoint")
    };
    let Checkpoint::Published {
        receipt,
        ready_backing,
        ..
    } = checkpoint
    else {
        panic!("delivery has not happened")
    };
    registry
        .finish_checkpoint(
            owner,
            claim,
            Checkpoint::Delivered {
                receipt,
                ready_backing,
                delivery: None,
            },
            None,
        )
        .unwrap();
    assert_eq!(registry.publication_counts_for_registry_test(), (0, 0));
    let Admission::AdvanceCheckpoint { claim, checkpoint } = registry.begin(owner) else {
        panic!("readiness follows delivery")
    };
    let Checkpoint::Delivered {
        receipt,
        ready_backing,
        ..
    } = checkpoint
    else {
        panic!("no second delivery")
    };
    let completion = ready_backing.complete(super::super::WorthQueryCompletedOutputDemand {
        authority: super::super::WorthQueryAcceptedOutputAuthority::Committed(receipt),
        readiness: crate::domain_computation::primary_graph::WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
    });
    registry
        .finish_checkpoint(owner, claim, Checkpoint::Ready(completion), None)
        .unwrap();
    assert!(matches!(registry.begin(owner), Admission::Ready(_)));
}

#[test]
fn refused_frame_custody_leaves_a_live_entry_to_deliver_once() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    let source = WorthQueryOutputDemandInterest::source_for_typed_frame_test();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let row = key("live-delivery-entry", 1, 1);
        let owner = admit(&registry, &row);
        let peer = admit(&registry, &row);
        publish(&registry, &owner, receipt);
        let frame = owner.real_typed_frame_for_registry_test(source);
        refuse_custody(&registry);
        frame();
        assert_eq!(registry.publication_counts_for_registry_test(), (1, 1));
        completes_once(&registry, &peer);
    });
}

#[test]
fn refused_frame_custody_leaves_a_stale_entry_to_rejoin_and_deliver_once() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    let source = WorthQueryOutputDemandInterest::source_for_typed_frame_test();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let stale = admit(&registry, &key("stale-delivery-entry", 1, 1));
        let row = key("stale-delivery-entry", 2, 1);
        let owner = admit(&registry, &row);
        // Opaque owner-issued transport is the lifetime fixture's declared
        // readmission; native certification is outside this release law.
        registry.state.lock().unwrap().records.get_mut(&row).unwrap().readmission_source = Some(Arc::new(
            super::super::source_readmission::RequiredOutputReadmission {
                source: Arc::new(()), producer: std::sync::OnceLock::new(),
                limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile::standard().limits(),
                retained_program_basis: None,
            }));
        publish(&registry, &owner, receipt);
        let frame = owner.real_typed_frame_for_registry_test(source);
        refuse_custody(&registry);
        frame();
        assert_eq!(registry.publication_counts_for_registry_test(), (1, 1));
        let (rejoined, _) = registry
            .rejoin_refreshed(&stale, &mut record_admission())
            .unwrap()
            .unwrap();
        drop(stale);
        completes_once(&registry, &rejoined);
    });
}
