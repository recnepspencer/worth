//! A retained unrelated row makes over-refunds visible in all three ledgers.
use super::super::{
    PerformedOutputObligation, PreparedReadyBacking, WorthQueryOutputCheckpoint,
    WorthQueryOutputProgress, WorthQueryPendingOutputDelivery,
};
use super::*;

pub(super) fn install(
    state: &mut DemandRegistryState,
    receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
) -> (WorthQueryOutputDemandKey, (usize, usize, usize)) {
    let key = key("refund-baseline", 1, 7);
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let backing = PreparedReadyBacking::prepare(state, &mut record_admission(), 0).unwrap();
    let output = WorthQueryOutputProgress::new(WorthQueryOutputCheckpoint::Published {
        receipt,
        delivery: WorthQueryPendingOutputDelivery::NoChange,
        ready_backing: backing,
    });
    // The receipt is real; this local lifetime fixture declares an interested
    // checkpoint and its already funded performed obligation directly.
    let mut row = record(occurrence(), DemandState::Output(output), 1);
    row.performed_obligations.push(PerformedOutputObligation {
        source_commit: commit,
        source: key.source.clone(),
    });
    state.obligation_reserved_bytes += row.obligation_reserved_bytes();
    state.records.insert(key.clone(), row);
    let member = state.prepare_required_member(&key).unwrap();
    state.install_required_member(member);
    let baseline = balances(state);
    assert!(baseline.0 > 0 && baseline.1 > 0 && baseline.2 > 0);
    (key, baseline)
}

pub(super) fn balances(state: &DemandRegistryState) -> (usize, usize, usize) {
    (
        state.required_reserved_bytes,
        state.obligation_reserved_bytes,
        state
            .required_custody_retained_bytes
            .load(std::sync::atomic::Ordering::Acquire),
    )
}
