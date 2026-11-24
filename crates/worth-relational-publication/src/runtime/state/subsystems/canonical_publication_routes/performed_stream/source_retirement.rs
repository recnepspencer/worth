//! A fixed cursor visits the actual original performed chain, without a roster.
use super::*;
pub struct RelationalPerformedSourceWalk {
    current: Option<Arc<PerformedPublicationNode>>,
}
impl PerformedPublicationStream {
    pub(crate) fn source_retirement_walk(&self) -> RelationalPerformedSourceWalk {
        RelationalPerformedSourceWalk {
            current: self.head.load_full(),
        }
    }
}
impl RelationalPerformedSourceWalk {
    pub fn next_envelope(
        &mut self,
    ) -> Option<Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>>{
        let node = self.current.take()?;
        self.current = node.previous.clone();
        Some(node.route.envelope.clone())
    }
}
