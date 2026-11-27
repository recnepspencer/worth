//! The portable image operation explicitly demands every selected cold subject.
use super::checkpoint_capture::CapturedCheckpointBasis;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::LineageSubsystem;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

impl CapturedCheckpointBasis {
    pub fn demand_memory_image(
        mut self,
        symbols: &RuntimeSymbolTable,
        lineage: &LineageSubsystem,
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
        let _mirror = self.partitions.demand_all(execution, symbols)?;
        for root in &self.branch_roots {
            root.demand_all_full(execution, symbols)?;
        }
        if let Some(directory) = &self.selected_directory {
            symbols.demand_symbol_ids(
                execution,
                directory
                    .manifest
                    .symbols
                    .iter()
                    .map(|page| contracts::symbols::data::symbol::Symbol(page.first as u32)),
            )?;
            for page in &directory.manifest.lineage {
                lineage.demand_node(
                    execution,
                    contracts::identity::data::primitives::LineageId(page.first),
                )?;
            }
        }
        self.lineage_event_leases = lineage.demand_all_event_pages(execution)?;
        self.lineage_events = lineage
            .published_rows()
            .into_iter()
            .map(|(event, publication_commit_id)| {
                worth_relational_lineage::internal_port::durability::data::lineage_event_pages::RelationalCheckpointLineageEventRow {
                    event,
                    publication_commit_id,
                }
            })
            .collect();
        self.lineage_event_pages_resident = true;
        if let Some(image) = self.index_original.take() {
            image.readmit_into_snapshot(execution, &mut self.index_state)?;
        }
        self.derived_index_checkpoint =
            contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
                |lane| {
                    super::super::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(
                        lane,
                        self.index_state
                            .retained_generations(lane, &self.retained_indexes),
                        self.index_state
                            .basis_checkpoint(lane, &self.retained_indexes),
                    )
                },
            )?;
        self.symbol_table = symbols.snapshot();
        self.lineage_nodes = lineage.nodes_snapshot();
        Ok(self)
    }
}
