//! Activate one original source grant at the first executed complete traversal.
use super::*;
use crate::domain_computation::primary_graph::source_facts::SourceFactReadDenial;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

impl RootPathSelectionBuilder {
    pub(super) fn finish(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<RootPathSelection, Cause> {
        if let Some(allocation) = &self.allocation {
            allocation.claim_tree_copy::<(EntityId, SourceFacts)>(execution, self.roots.len())?;
        }
        let mut roots = BTreeMap::new();
        for (root, source) in self.roots {
            roots.insert(root, source.finish(execution)?);
        }
        let set_source = self
            .set_source
            .map(|source| source.finish(execution))
            .transpose()?;
        Ok(RootPathSelection {
            roots,
            set_source,
            allocation: self.allocation,
        })
    }

    pub(super) fn activate_source_allocation(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        frontier: &mut Frontier,
    ) -> Result<(), SourceFactReadDenial> {
        if self.allocation.is_some() {
            return Ok(());
        }
        let allocation =
            SourceFactsAllocation::open(execution).map_err(SourceFactReadDenial::Execution)?;
        for sources in [&mut self.roots, frontier] {
            allocation
                .claim_tree_copy::<(EntityId, SourceFactsBuilder)>(execution, sources.len())
                .map_err(SourceFactReadDenial::Execution)?;
            for source in sources.values_mut() {
                source.protect(execution, &allocation)?;
            }
        }
        if let Some(source) = &mut self.set_source {
            source.protect(execution, &allocation)?;
        }
        self.allocation = Some(allocation);
        Ok(())
    }

    pub(super) fn admit_frontier_entry(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        len: usize,
    ) -> Result<(), Cause> {
        if let Some(allocation) = &self.allocation {
            allocation.claim_tree::<(EntityId, SourceFactsBuilder)>(execution, len)?;
        }
        Ok(())
    }

    pub(super) fn admit_frontier_keys(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        len: usize,
    ) -> Result<(), Cause> {
        if let Some(allocation) = &self.allocation {
            allocation.claim_tree_copy::<(EntityId, ())>(execution, len)?;
        }
        Ok(())
    }
}
