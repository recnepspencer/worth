//! Read-only retained Native writer attribution on a performed receipt.
use super::CommitPublication;

impl CommitPublication {
    /// Original applied source effects, before synchronous producers appended
    /// their actual writes. This is descriptive receipt custody, never a grant.
    pub fn authored_source_effects(
        &self,
    ) -> Option<&crate::mvcc::RelationalCoupledAuthoredSourceEffects> {
        self.source_effects.as_ref()
    }
}

impl CommitPublication {
    /// The actual sources of a coupled ordinary Edit, prefix first and then
    /// every consumed stage in apply order. Corrective prefixes retain their
    /// genuine event instead. Descriptive receipt data, never authority.
    pub fn coupled_authored_stage_sources(&self) -> Option<&[crate::identity::data::EntityId]> {
        match &self.envelope.coupled_authored_publication.as_ref()?.sources {
            crate::history::data::CanonicalCoupledAuthoredSources::Edit { sources } => {
                Some(sources)
            }
            _ => None,
        }
    }
}
