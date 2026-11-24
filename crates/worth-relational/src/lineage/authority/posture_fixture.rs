//! Input to the Lineage finalization fixture.
use crate::identity::data::LineageId;
use crate::lineage::data::{
    InstalledPersistentNameSchemeRegistry, LineageBasisView, LineagePostureClaim, LineageSubject,
    PersistentNameHolder, PersistentNameIdentity,
};
use std::collections::BTreeMap;

#[cfg(test)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineageCommitPostureInput {
    pub scheme_registry: InstalledPersistentNameSchemeRegistry,
    pub live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    pub live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    /// Previously retired persistent names and their most recent retired
    /// `(LineageSubject, LineageId)` at the commit's basis.
    pub retired_lineages: BTreeMap<PersistentNameIdentity, (LineageSubject, LineageId)>,
    pub postures: Vec<LineagePostureClaim>,
}

#[cfg(test)]
impl LineageBasisView for LineageCommitPostureInput {
    fn live_holder(&self, id: &PersistentNameIdentity) -> Option<&PersistentNameHolder> {
        self.live_holders.get(id)
    }

    fn live_lineage(&self, holder: &PersistentNameHolder) -> Option<LineageId> {
        self.live_lineages.get(holder).copied()
    }

    fn retired_lineage(&self, id: &PersistentNameIdentity) -> Option<(LineageSubject, LineageId)> {
        self.retired_lineages.get(id).copied()
    }

    fn holder_name(&self, holder: &PersistentNameHolder) -> Option<PersistentNameIdentity> {
        self.live_holders
            .iter()
            .find_map(|(name, live)| (live == holder).then_some(*name))
    }
}
