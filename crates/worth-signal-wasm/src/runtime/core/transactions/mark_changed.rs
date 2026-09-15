use worth_signal::facade::{Aspect, ChangedRegion, SignalError};

use crate::boundary::errors::WorthSignalJsError;
use crate::runtime::summaries::RunSummary;

use super::super::aspects::{bump_aspects, normalize_aspects};
use super::super::debug::{perf_now_ms, wasm_debug};
use super::super::{RuntimeCore, DEFAULT_ASPECT};
use crate::recipe::model::WasmAspectId;

impl RuntimeCore {
    pub fn mark_changed_with_regions(
        &mut self,
        id: &str,
        changed_regions: Vec<ChangedRegion>,
    ) -> Result<RunSummary, WorthSignalJsError> {
        self.mark_changed_with_regions_on_aspects(id, changed_regions, vec![DEFAULT_ASPECT])
    }

    pub fn mark_changed_on_aspects(
        &mut self,
        id: &str,
        aspect_ids: Vec<WasmAspectId>,
    ) -> Result<RunSummary, WorthSignalJsError> {
        self.mark_changed_with_regions_for_aspect_ids(id, Vec::new(), aspect_ids)
    }

    pub fn mark_changed_with_regions_for_aspect_ids(
        &mut self,
        id: &str,
        changed_regions: Vec<ChangedRegion>,
        aspect_ids: Vec<WasmAspectId>,
    ) -> Result<RunSummary, WorthSignalJsError> {
        let aspects = normalize_aspects(&aspect_ids)?;
        self.mark_changed_with_regions_on_aspects(id, changed_regions, aspects)
    }

    pub fn mark_changed_with_regions_on_aspects(
        &mut self,
        id: &str,
        changed_regions: Vec<ChangedRegion>,
        aspects: Vec<Aspect>,
    ) -> Result<RunSummary, WorthSignalJsError> {
        let node = self.node_for_id(id)?;
        let started_at = perf_now_ms();
        let previous = self.lock_store()?.clone();
        let store = self.store.clone();
        let evaluator = self.evaluator();
        let standing_demand = self.standing_demand_nodes();

        let branch = self.runtime.current_branch();
        let basis = self.native_branch_basis(branch)?;
        let result = self
            .runtime
            .advance_signal_branch(&mut self.store, &basis, move |tx| {
                {
                    let mut locked = store
                        .lock()
                        .map_err(|_| SignalError::internal("runtime store mutex poisoned"))?;
                    let source = locked.sources.get_mut(id).ok_or_else(|| {
                        SignalError::invalid_input(format!("unknown source `{id}`"))
                    })?;
                    source.version = bump_aspects(source.version, &aspects);
                }

                for aspect in &aspects {
                    tx.mark_changed_with_regions(node, *aspect, &changed_regions)?;
                }
                tx.evaluate_dirty(&evaluator)?;
                tx.evaluate_demand(&evaluator, &standing_demand)?;
                Ok(())
            });

        match result {
            Ok(outcome) => {
                let (_, result) = outcome.into_parts();
                self.apply_pending_callback_dependency_patches()?;
                self.advance_current_authored_graph_generation();
                let active_branch_id = self.runtime.current_branch().id.0;
                self.branch_states
                    .insert(active_branch_id, self.snapshot_branch_state());
                wasm_debug(format!(
                    "[worth-signals-wasm] tx:regions-done touched={} evaluated={} elapsed_ms={:.1}",
                    result.touched_nodes,
                    result.evaluation_summary.nodes_evaluated,
                    perf_now_ms() - started_at
                ));
                Ok(RunSummary {
                    touched_nodes: result.touched_nodes,
                    nodes_evaluated: result.evaluation_summary.nodes_evaluated,
                    nodes_recomputed: result.evaluation_summary.nodes_recomputed,
                    nodes_suppressed: result.evaluation_summary.nodes_suppressed,
                    plans_built: result.evaluation_summary.plans_built,
                    stages_executed: result.evaluation_summary.stages_executed,
                    total_nanos: result.timing.total_nanos.to_string(),
                    evaluation_nanos: result.timing.evaluation_nanos.to_string(),
                    commit_nanos: result.timing.commit_nanos.to_string(),
                })
            }
            Err(err) => {
                wasm_debug(format!(
                    "[worth-signals-wasm] tx:regions-error elapsed_ms={:.1} denial={:?}",
                    perf_now_ms() - started_at,
                    err
                ));
                self.restore_store(previous)?;
                Err(WorthSignalJsError::invalid_input(format!(
                    "Signal branch advance denied: {err:?}"
                )))
            }
        }
    }
}
