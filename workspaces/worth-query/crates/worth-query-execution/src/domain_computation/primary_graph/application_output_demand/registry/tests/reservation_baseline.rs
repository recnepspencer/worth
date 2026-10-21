//! An unrelated retained publication remains present through neighboring cleanup.
use super::super::{
    PerformedOutputObligation, PreparedReadyBacking, WorthQueryOutputCheckpoint,
    WorthQueryOutputProgress, WorthQueryPendingOutputDelivery,
};
use super::*;

pub(super) fn install(
    state: &mut DemandRegistryState,
    receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
) -> WorthQueryOutputDemandKey {
    let key = key("refund-baseline", 1, 7);
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let backing = PreparedReadyBacking::new();
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
    state.records.insert(key.clone(), row);
    let member = state.prepare_required_member(&key).unwrap();
    state.install_required_member(member);
    key
}
