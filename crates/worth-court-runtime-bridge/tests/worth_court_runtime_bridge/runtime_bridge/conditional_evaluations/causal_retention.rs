//! Public execution and reentry preserve readonly original custody without a reader.
use crate::conditional_evaluations::native_absence::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_runtime_bridge::facade::*;
#[test]
fn causal_observation_preserves_native_custody_without_reader_or_second_performed_receipt() {
    let contacts = Arc::new(Contacts::default());
    let readers = Arc::new(AtomicUsize::new(0));
    let (owner, lowering) = owner(Arc::clone(&contacts), Arc::clone(&readers));
    let probe = owner.conditional_lifecycle_probe();
    let bridge_baseline = probe
        .bridge_conditional_retention()
        .unwrap()
        .retained_bytes();
    let serial = worth_execution::SerialRequest::from_policy(
        &BridgeExecutionPolicyBaseline::operational().request_policy(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let source = snapshot();
    let basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let evidence = owner
        .execute(
            execution,
            &basis,
            BridgeConditionalExecutionRequest {
                lowering: &lowering,
                query_binding_identity: "query-binding",
                query_capability_identity: 1,
                snapshot_identity: "snapshot-label",
                truth_branch_identity: None,
                bridge_snapshot_identity: Some(&source),
                execution_identity: "execution",
                attempt: 1,
            },
            &mut (),
        )
        .unwrap();
    let causal = evidence.retain_causal_observation();
    let last_causal = causal.clone();
    assert!(causal.same_observation(&last_causal));
    let signal = causal.performed_signal_invalidation().unwrap().clone();
    let performed = evidence.performed_signal_invalidation().unwrap();
    assert!(signal.same_observation(&performed.retain_causal_observation()));
    let seed = evidence.retain_for_reentry();
    let reentered = owner
        .reenter_retained_conditional_decision(BridgeConditionalDecisionReentryRequest {
            seed: &seed,
            lowering: &lowering,
            query_binding_identity: "fresh-binding",
            query_capability_identity: 2,
            snapshot_identity: "snapshot-label",
            bridge_snapshot_identity: Some(&source),
        })
        .unwrap();
    assert!(reentered.performed_signal_invalidation().is_none());
    assert!(causal.same_observation(&reentered.retain_causal_observation()));
    assert_eq!(readers.load(Ordering::SeqCst), 1);
    drop((evidence, reentered));
    assert_eq!(
        readers.load(Ordering::SeqCst),
        1,
        "operational seed still holds its reader"
    );
    drop(seed);
    assert_eq!(
        readers.load(Ordering::SeqCst),
        0,
        "causal handles must not pin the admitted reader"
    );
    assert_eq!(causal.bridge_snapshot_identity(), Some(&source));
    assert_eq!(causal.semantic_observations().len(), 1);
    assert_eq!(contacts.reads.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.predicates.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.computes.load(Ordering::SeqCst), 1);
    let retained = probe
        .bridge_conditional_retention()
        .unwrap()
        .retained_bytes();
    assert!(retained > bridge_baseline);
    drop(causal);
    assert_eq!(
        probe
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        retained
    );
    drop(last_causal);
    assert_eq!(
        probe
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        bridge_baseline
    );
    let signal_retained = probe
        .signal_conditional_retention()
        .unwrap()
        .retained_bytes();
    drop(signal);
    assert!(
        probe
            .signal_conditional_retention()
            .unwrap()
            .retained_bytes()
            < signal_retained,
        "the last readonly Signal share releases the original receipt reservation"
    );
    drop((basis, lowering, owner));
    assert!(probe.bridge_conditional_retention().is_none());
    assert_eq!(readers.load(Ordering::SeqCst), 0);
}
