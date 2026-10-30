//! Original-paid recorded dispositions retain the actual visited structural cut.
use super::super::ordered_storage as maps;
use super::*;
use worth_execution::{ExecutionOrderedMap as Map, ExecutionRequest};
use worth_proof::{
    ManagedCoverageDisposition as Disposition, ManagedCoverageFrontier, ManagedCoverageRegion,
    ManagedCoverageStep,
};

pub(in crate::domain_computation::primary_graph) struct RecordedFrontier(Map<usize, Disposition>);
impl RecordedFrontier {
    pub(in crate::domain_computation::primary_graph) fn new() -> Self {
        Self(Map::default())
    }
    pub(in crate::domain_computation::primary_graph) fn record(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        node: usize,
        disposition: Disposition,
        control: &Control<'_>,
    ) -> Result<(), Denial> {
        maps::lookup(control, self.0.len(), size_of::<usize>())?;
        if self.0.contains_key(&node) {
            return Err(control.refuse(Stop::IncompleteCoverage).into());
        }
        maps::insert(&mut self.0, &node, &disposition, execution, control).map_err(Into::into)
    }
    pub(super) fn verify(
        &self,
        issuer: &worth_proof::ManagedIndexCoverageIssuer,
        tree: &SourceTree,
        pruning: bool,
        control: &Control<'_>,
    ) -> Result<worth_proof::AdmittedManagedIndexCoverage, Denial> {
        let mut steps = self
            .0
            .iter()
            .map(|(node, disposition)| ManagedCoverageStep {
                node: *node,
                disposition: *disposition,
            });
        let mut region = |node| {
            let region = tree.region(node, control).ok()?;
            Some(ManagedCoverageRegion {
                parent: tree.parent(node, control).ok()?,
                children: region.children,
                has_source: region.source.is_some(),
            })
        };
        let mut disposition = |node| {
            maps::lookup(control, self.0.len(), size_of::<usize>()).ok()?;
            self.0.get(&node).copied()
        };
        issuer
            .verify(ManagedCoverageFrontier {
                root: tree.root(),
                recorded_regions: self.0.len(),
                allow_pruning: pruning,
                steps: &mut steps,
                region: &mut region,
                disposition: &mut disposition,
                checkpoint: &mut || control.checkpoint(1).is_ok(),
            })
            .map_err(|cause| control.refuse(Stop::Coverage(cause)).into())
    }
    pub(super) fn admits_slots(
        &self,
        tree: &SourceTree,
        slots: &[Slot],
        matched: usize,
        control: &Control<'_>,
    ) -> Result<(), Denial> {
        if matched != slots.len() {
            return Err(control.refuse(Stop::IncompleteCoverage).into());
        }
        let mut previous = None;
        for slot in slots {
            control.checkpoint(1)?;
            if previous.is_some_and(|before| before >= *slot) {
                return Err(control.refuse(Stop::IncompleteCoverage).into());
            }
            previous = Some(*slot);
            let node = tree
                .source_node(*slot, control)?
                .ok_or_else(|| control.refuse(Stop::ForeignSourceSlot))?;
            maps::lookup(control, self.0.len(), size_of::<usize>())?;
            if self.0.get(&node) != Some(&Disposition::Matched) {
                return Err(control.refuse(Stop::IncompleteCoverage).into());
            }
        }
        control.poll().map_err(Into::into)
    }
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
}
