use worth_signal::facade::history::{
    RuntimeSnapshot as SignalSnapshotV1, SnapshotRestoreIntent,
};
use worth_signal::facade::SignalGraph;

fn main() {
    let _capture: fn(&mut SignalGraph) -> SignalSnapshotV1 = SignalGraph::capture_snapshot;
    let _restore: fn(&mut SignalGraph, &SignalSnapshotV1) -> _ = SignalGraph::restore_snapshot;
    let _restore_with_intent: fn(&mut SignalGraph, &SignalSnapshotV1, SnapshotRestoreIntent) -> _ =
        SignalGraph::restore_snapshot_with_intent;
    let _authority_graph: fn(&SignalSnapshotV1) -> _ = SignalSnapshotV1::authority_graph;
}
