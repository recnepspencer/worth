//! Exact Source structure stays inside the existing provider/root custody pair.
mod admission;
mod capture;
mod maps;
mod region;
mod transition;
use super::{
    source_rows::{SourceImageView, WorthQueryManagedIndexSourceSlot as Slot},
    WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexSourceChange as Change,
    WorthQueryManagedIndexStop as Stop,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
pub use region::WorthQueryManagedIndexNodeRegion;
use region::{Edit, Region};
use worth_execution::{ExecutionOrderedMap as Map, ExecutionRequest};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SourceTree {
    regions: Map<usize, Region>,
    parents: Map<usize, usize>,
    sources: Map<Slot, usize>,
    owner: WorthQueryGraphWorkSessionIdentity,
}
impl SourceTree {
    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.regions.len()
    }
    pub(in crate::domain_computation::primary_graph) fn source_len(&self) -> usize {
        self.sources.len()
    }
    pub(in crate::domain_computation::primary_graph) fn region(
        &self,
        ordinal: usize,
        control: &Control<'_>,
    ) -> Result<Region, Stop> {
        maps::lookup(control, self.regions.len(), size_of::<usize>())?;
        self.regions
            .get(&ordinal)
            .copied()
            .ok_or_else(|| control.refuse(Stop::IncompleteTraversal))
    }
    pub(in crate::domain_computation::primary_graph) fn parent(
        &self,
        ordinal: usize,
        control: &Control<'_>,
    ) -> Result<Option<usize>, Stop> {
        maps::lookup(control, self.parents.len(), size_of::<usize>())?;
        Ok(self.parents.get(&ordinal).copied())
    }
    fn empty(owner: WorthQueryGraphWorkSessionIdentity) -> Self {
        Self {
            regions: Map::default(),
            parents: Map::default(),
            sources: Map::default(),
            owner,
        }
    }
}

pub(super) use capture::{qualify, Step};

/// Record only actual physical writes; repeated writes do not grow edit scratch.
pub(super) fn record_change(
    changed: &mut Map<usize, ()>,
    ordinal: usize,
    execution: &ExecutionRequest<'_, '_>,
    control: &Control<'_>,
) -> Result<(), Stop> {
    maps::lookup(control, changed.len(), size_of::<usize>())?;
    if !changed.contains_key(&ordinal) {
        maps::insert(changed, &ordinal, &(), execution, control)?;
    }
    Ok(())
}
