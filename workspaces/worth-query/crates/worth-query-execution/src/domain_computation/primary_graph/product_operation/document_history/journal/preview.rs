//! Only the annotation owner can obtain a completed node's real source custody.
use super::super::{HistoryModelIdentity, WorthQueryHistoryPreview};
use super::*;

impl<S> HistoryJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph) fn owns_node(
        &self,
        id: WorthQueryHistoryNodeId,
    ) -> bool {
        id.document == self.incarnation
    }

    pub(in crate::domain_computation::primary_graph) fn preview(
        &self,
        id: WorthQueryHistoryNodeId,
        model: HistoryModelIdentity,
    ) -> Result<WorthQueryHistoryPreview, Denial> {
        retain_preview(
            self.incarnation,
            &self.nodes,
            &self.retained,
            self.observation.as_ref(),
            id,
            model,
        )
    }
}

fn retain_preview(
    document: DocumentIncarnation,
    nodes: &BTreeMap<u64, HistoryNode>,
    retained: &BTreeMap<u64, RetainedHistorySource>,
    observation: Option<&EnrollmentSink>,
    id: WorthQueryHistoryNodeId,
    model: HistoryModelIdentity,
) -> Result<WorthQueryHistoryPreview, Denial> {
    if id.document != document {
        return Err(Denial::ForeignHistoryNode);
    }
    if let Some(observation) = observation {
        observation.access(Access::Node);
    }
    let Some(HistoryNode {
        binding: NodeBinding::Completed(node),
        ..
    }) = nodes.get(&id.ordinal)
    else {
        return Err(Denial::HistoryNodeUnavailable);
    };
    Ok(WorthQueryHistoryPreview {
        step: node.step.clone(),
        model,
        source: Arc::clone(
            retained
                .get(&id.ordinal)
                .and_then(|entry| entry.source.as_ref())
                .ok_or(Denial::HistoryNodeExcluded)?,
        ),
        root_posture: node.root_posture,
    })
}
