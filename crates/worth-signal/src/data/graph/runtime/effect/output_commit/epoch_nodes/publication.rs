//! Installation of a fully prepared selected-node epoch.
use super::{
    EpochPublishedProducer, PreparedEpochNodeEdits, RetainedNodeEditOutcome, SelectedNodeDraft,
};
use crate::data::graph::SignalGraph;

impl PreparedEpochNodeEdits {
    pub(in crate::data::graph::runtime::effect::output_commit) fn preconditions_hold(
        &self,
        graph: &SignalGraph,
    ) -> bool {
        self.retained
            .as_ref()
            .is_none_or(|roots| roots.storage_preconditions_hold(&graph.arena))
            && self
                .ordinary
                .as_ref()
                .is_none_or(|draft| draft.node_count == graph.arena.hot.len())
    }

    pub(in crate::data::graph::runtime::effect::output_commit) fn publish(
        self,
        graph: &mut SignalGraph,
    ) -> Vec<EpochPublishedProducer> {
        let published = match (self.ordinary, self.retained) {
            (Some(draft), None) => {
                for (index, payload) in draft.selected {
                    match payload {
                        SelectedNodeDraft::ProducerFull(payload) => {
                            graph.arena.hot.replace_discard(index, Some(payload.hot));
                            graph.arena.warm.replace_discard(index, payload.warm);
                            graph.arena.cold.replace_discard(index, payload.cold);
                        }
                        SelectedNodeDraft::ConsumerOperational(payload) => {
                            graph.arena.hot.replace_discard(index, Some(payload.hot));
                            graph.arena.warm.replace_discard(index, payload.warm);
                        }
                    }
                }
                draft.published
            }
            (None, Some(roots)) => match roots.install(&mut graph.arena) {
                RetainedNodeEditOutcome::Installed { output, .. } => output,
                RetainedNodeEditOutcome::Rejected { .. } => {
                    unreachable!("exclusive epoch publication")
                }
            },
            _ => unreachable!("one epoch node representation"),
        };
        self.waiters.publish(graph);
        published
    }
}
