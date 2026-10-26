//! The portable image operation explicitly demands every selected cold subject.
use super::checkpoint_capture::CapturedCheckpointBasis;
use crate::durability::data::RelationalRecoveryStop;
use crate::runtime::RelationalRuntime;

impl CapturedCheckpointBasis {
    pub(super) fn demand_memory_image(
        mut self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalRecoveryStop> {
        // Hold the selection's bodies through image construction. Demand the exact
        // captured roots and mirror rather than consulting a later branch head.
        self.envelopes = self
            .envelope_cells
            .iter()
            .map(|cell| cell.demand_positioned(execution, 1 << 30))
            .collect::<Result<_, _>>()?;
        self.envelopes.sort_by_key(|commit| commit.position());
        let _mirror = self
            .partitions
            .demand_all(execution, &runtime.services.symbols)?;
        for root in &self.branch_roots {
            root.demand_all_full(execution, &runtime.services.symbols)?;
        }
        if let Some(directory) = &self.selected_directory {
            runtime.services.symbols.demand_symbol_ids(
                execution,
                directory
                    .manifest
                    .symbols
                    .iter()
                    .map(|page| crate::symbols::data::Symbol(page.first as u32)),
            )?;
            for page in &directory.manifest.lineage {
                runtime
                    .lineage
                    .demand_node(execution, crate::identity::data::LineageId(page.first))?;
            }
        }
        self.symbol_table = runtime.services.symbols.snapshot();
        self.lineage_nodes = runtime.lineage_access().resident_nodes_snapshot();
        Ok(self)
    }
}
