use crate::data::handle::NodeId;
use crate::logic::transaction::patch_buffer::SparsePatchBuffer;

use super::transaction_types::GraphPatchRollbackDelta;
use super::SignalTransaction;

impl<D, I, E, Ctx, T> SignalTransaction<'_, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn inject_stale_graph_patch_rollback_packet_for_test(&mut self, node: NodeId) {
        let mut patches = SparsePatchBuffer::new();
        patches
            .stage_original(self.graph, node)
            .expect("live rollback target stages");
        self.graph
            .unregister_node(node)
            .expect("rollback target is removed after packet capture");
        self.rollback_packets
            .stage_graph_patches(GraphPatchRollbackDelta { patches })
            .expect("fresh transaction has no staged graph-patch rollback packet");
    }
}

#[test]
fn public_rollback_preserves_poisoned_diagnostic_outcome_for_packet_failure() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let target = graph.create_node();
    let mut runtime =
        crate::logic::transaction::SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    let mut context = ();
    let mut transaction = runtime.begin(&mut context);
    transaction.inject_stale_graph_patch_rollback_packet_for_test(target);

    let result = transaction.rollback().unwrap();
    assert_eq!(result.outcome, super::TransactionOutcome::Poisoned);
}
