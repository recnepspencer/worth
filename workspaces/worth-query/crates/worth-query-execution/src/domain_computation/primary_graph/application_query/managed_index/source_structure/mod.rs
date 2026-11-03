//! Exact Source structure stays inside the existing provider/root custody pair.
mod admission;
mod ancestry;
mod capture;
pub mod cold;
mod cold_columns;
mod cold_postings;
use super::nodes::pages::{NodePages, PageEdit};
use crate::domain_computation::primary_graph::access_strategy::ordered_storage as maps;
mod region;
mod summary;
mod summary_context;
mod summary_frontier;
mod summary_sources;
pub(super) use summary_context::SummaryContext;
mod transition;
use super::{
    source_rows::{SourceImageView, WorthQueryManagedIndexSourceSlot as Slot},
    WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexSourceChange as Change,
    WorthQueryManagedIndexStop as Stop,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
pub use region::WorthQueryManagedIndexNodeRegion;
use region::{Edit, Region};
pub use summary_sources::WorthQueryManagedIndexSummarySources;
use worth_execution::{ExecutionOrderedMap as Map, ExecutionRequest};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SourceTree {
    root: Option<usize>,
    regions: NodePages<Region>,
    parents: NodePages<usize>,
    region_count: usize,
    parent_count: usize,
    sources: Map<Slot, usize>,
    owner: WorthQueryGraphWorkSessionIdentity,
}
impl SourceTree {
    pub(in crate::domain_computation::primary_graph) fn source_node(
        &self,
        slot: Slot,
        control: &Control<'_>,
    ) -> Result<Option<usize>, Stop> {
        maps::lookup(control, self.sources.len(), size_of::<Slot>())?;
        Ok(self.sources.get(&slot).copied())
    }
    pub(in crate::domain_computation::primary_graph) fn root(&self) -> Option<usize> {
        self.root
    }
    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.region_count
    }
    pub(in crate::domain_computation::primary_graph) fn source_len(&self) -> usize {
        self.sources.len()
    }
    pub(in crate::domain_computation::primary_graph) fn region(
        &self,
        ordinal: usize,
        control: &Control<'_>,
    ) -> Result<Region, Stop> {
        self.region_at(ordinal, control)?
            .ok_or_else(|| control.refuse(Stop::IncompleteTraversal))
    }
    fn region_at(&self, ordinal: usize, control: &Control<'_>) -> Result<Option<Region>, Stop> {
        control.checkpoint(1)?;
        self.regions
            .get(ordinal, control)
            .map(|region| region.copied())
    }
    pub(in crate::domain_computation::primary_graph) fn parent(
        &self,
        ordinal: usize,
        control: &Control<'_>,
    ) -> Result<Option<usize>, Stop> {
        control.checkpoint(1)?;
        self.parents
            .get(ordinal, control)
            .map(|parent| parent.copied())
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
