//! A fixed cursor visits the actual original performed chain, without a roster.
use super::*;
pub(crate) struct RelationalPerformedSourceWalk {
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
    pub(crate) fn next(
        &mut self,
    ) -> Option<Arc<crate::history::RelationalCommitEnvelopeResidency>> {
        let node = self.current.take()?;
        self.current = node.previous.clone();
        Some(node.route.envelope.clone())
    }
}
