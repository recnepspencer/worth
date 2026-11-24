//! A selected head lends a physical symbol update for one publication.
use super::{
    Head, RegionSymbolChanges, RelationalRegionSymbolAdmissionIdentity, RelationalResidencyLease,
};
use crate::symbols::data::Symbol;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RegionSymbolPublication {
    pub(super) publication: SymbolPublicationKey,
    pub(super) changes: Arc<RegionSymbolChanges>,
}

impl RegionSymbolPublication {
    pub fn required_symbols(&self) -> impl Iterator<Item = Symbol> + '_ {
        self.changes.owned.ids.iter().copied()
    }
}

impl RelationalResidencyLease<Head> {
    pub fn symbol_publication(
        &self,
        publication: &SymbolPublicationKey,
    ) -> Option<RegionSymbolPublication> {
        let changes = self
            .cell()
            .symbol_inheritance
            .get()
            .map(|basis| basis.changes.clone())
            .or_else(|| {
                self.resident_region()
                    .symbol_requirements
                    .get()
                    .and_then(|seed| seed.changes())
            })?;
        Some(RegionSymbolPublication {
            publication: publication.clone(),
            changes,
        })
    }
}

/// Retained cells carry only the accepted dictionary predicate, never a root
/// or canonical envelope. That authority remains with the publication scope.
#[derive(Debug, Clone)]
pub struct SymbolPublicationKey(Arc<()>);
impl SymbolPublicationKey {
    pub fn new() -> Self {
        Self(Arc::new(()))
    }

    pub fn identity(&self) -> RelationalRegionSymbolAdmissionIdentity {
        RelationalRegionSymbolAdmissionIdentity::publication(&self.0)
    }
}
