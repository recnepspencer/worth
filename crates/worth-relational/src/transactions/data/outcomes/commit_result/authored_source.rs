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
    /// The authored sources an edit envelope recorded, in apply order: its
    /// prefix's source, then each later authored stage's. `None` for every
    /// other commit, including a single coupled edit. Descriptive only.
    pub fn coupled_authored_stage_sources(&self) -> Option<&[crate::identity::data::EntityId]> {
        match &self.envelope.coupled_authored_publication.as_ref()?.sources {
            crate::history::data::CanonicalCoupledAuthoredSources::Stages(sources) => Some(sources),
            _ => None,
        }
    }
}
