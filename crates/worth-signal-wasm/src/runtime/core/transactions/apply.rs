use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use worth_signal::facade::SignalTransaction;
use worth_signal::facade::{specialist::EvaluationVerdict, SignalError};

use crate::boundary::errors::WorthSignalJsError;
use crate::runtime::summaries::RunSummary;

use super::super::debug::{perf_now_ms, wasm_debug};
use super::super::keyed_families::set_rgba_signal_value;
use super::super::state::{DenseGridFamily, PendingCallbackDependencyPatch, SharedStore};
use super::super::RuntimeCore;
use super::changes::SetChange;
use crate::recipe::model::TransactionOp;

impl RuntimeCore {
    pub fn mark_keyed_changed_with_regions(
        &mut self,
        family_id: &str,
        key: &str,
        changed_regions: Vec<worth_signal::facade::ChangedRegion>,
    ) -> Result<RunSummary, WorthSignalJsError> {
        let id = self.ensure_source_key(family_id, key, None)?;
        self.mark_changed_with_regions(&id, changed_regions)
    }

    pub fn mark_keyed_changed_on_aspects(
        &mut self,
        family_id: &str,
        key: &str,
        aspect_ids: Vec<crate::recipe::model::WasmAspectId>,
    ) -> Result<RunSummary, WorthSignalJsError> {
        let id = self.ensure_source_key(family_id, key, None)?;
        self.mark_changed_on_aspects(&id, aspect_ids)
    }

    pub fn apply_transaction(
        &mut self,
        ops: Vec<TransactionOp>,
    ) -> Result<RunSummary, WorthSignalJsError> {
        let started_at = perf_now_ms();
        wasm_debug(format!("[worth-signals-wasm] tx:start ops={}", ops.len()));
        let previous = self.lock_store()?.clone();
        let changes = self.collect_changes(&ops)?;
        wasm_debug(format!(
            "[worth-signals-wasm] tx:collect-done changes={} elapsed_ms={:.1}",
            changes.len(),
            perf_now_ms() - started_at
        ));
        let store = self.store.clone();
        let dense_grids = self.dense_grids.clone();
        let evaluator = self.evaluator();
        let standing_demand = self.standing_demand_nodes();
        let committed_dependency_patches = Arc::new(Mutex::new(
            None::<(Vec<PendingCallbackDependencyPatch>, u64)>,
        ));
        let committed_dependency_patches_for_tx = committed_dependency_patches.clone();

        let branch = self.runtime.current_branch();
        let basis = self.native_branch_basis(branch)?;
        let result = self
            .runtime
            .advance_signal_branch(&mut self.store, &basis, move |tx| {
                wasm_debug("[worth-signals-wasm] tx:apply-start");
                apply_set_changes(tx, &store, &dense_grids, &changes)?;

                wasm_debug("[worth-signals-wasm] tx:evaluate-dirty-start");
                tx.evaluate_dirty(&evaluator)?;
                wasm_debug("[worth-signals-wasm] tx:evaluate-dirty-done");
                // Computed recipes are on-demand; watchers and published outputs
                // are demand. Recompute the demanded nodes this change reaches so
                // commit delivers them and the committed truth includes them.
                tx.evaluate_demand(&evaluator, &standing_demand)?;
                let (pending, runtime_read_breadth) =
                    apply_pending_dependency_patches_in_transaction(tx, &store)?;
                *committed_dependency_patches_for_tx.lock().map_err(|_| {
                    SignalError::internal("dependency patch receipt mutex poisoned")
                })? = Some((pending, runtime_read_breadth));
                Ok(())
            });

        match result {
            Ok(outcome) => {
                let (_, result) = outcome.into_parts();
                let (pending, runtime_read_breadth) = committed_dependency_patches
                    .lock()
                    .map_err(|_| {
                        WorthSignalJsError::internal(
                            "dependency patch receipt mutex poisoned".to_string(),
                        )
                    })?
                    .take()
                    .unwrap_or_default();
                self.record_committed_callback_dependency_patches(pending, runtime_read_breadth)?;
                self.advance_current_authored_graph_generation();
                let active_branch_id = self.runtime.current_branch().id.0;
                self.branch_states
                    .insert(active_branch_id, self.snapshot_branch_state());
                self.notify_diagnostics_subscribers();
                wasm_debug(format!(
                    "[worth-signals-wasm] tx:done touched={} evaluated={} elapsed_ms={:.1}",
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
                    "[worth-signals-wasm] tx:error elapsed_ms={:.1} denial={:?}",
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

    pub fn evaluate_dirty(&mut self) -> Result<RunSummary, WorthSignalJsError> {
        let evaluator = self.evaluator();
        let report = self
            .runtime
            .evaluate_dirty(&self.store, &evaluator)
            .map_err(WorthSignalJsError::from)?;
        self.notify_diagnostics_subscribers();
        Ok(RunSummary {
            touched_nodes: report.task_count,
            nodes_evaluated: report.tasks_executed,
            nodes_recomputed: report
                .stages
                .iter()
                .flat_map(|stage| stage.task_records.iter())
                .filter(|record| matches!(record.verdict, Some(EvaluationVerdict::Recomputed)))
                .count() as u32,
            nodes_suppressed: report.tasks_with_suppressed_propagation,
            plans_built: 1,
            stages_executed: report.stage_count,
            total_nanos: (report.execution_snapshot_nanos
                + report.stage_precompute_nanos
                + report.stage_apply_nanos
                + report.semantic_finalize_nanos)
                .to_string(),
            evaluation_nanos: (report.stage_precompute_nanos + report.stage_apply_nanos)
                .to_string(),
            commit_nanos: report.semantic_finalize_nanos.to_string(),
        })
    }
}

pub(in crate::runtime::core) fn apply_set_changes(
    tx: &mut SignalTransaction<'_, (), (), (), SharedStore, ()>,
    store: &SharedStore,
    dense_grids: &BTreeMap<String, Arc<DenseGridFamily>>,
    changes: &[SetChange],
) -> Result<(), SignalError> {
    let mut locked = store
        .lock()
        .map_err(|_| SignalError::internal("runtime store mutex poisoned"))?;
    for change in changes {
        match change {
            SetChange::Source {
                id,
                value,
                node,
                changed_regions,
                aspects,
            } => {
                let source = locked
                    .sources
                    .get_mut(id)
                    .ok_or_else(|| SignalError::invalid_input(format!("unknown source `{id}`")))?;
                source.value = value.clone();
                source.version = super::super::aspects::bump_aspects(source.version, aspects);
                for aspect in aspects {
                    if changed_regions.is_empty() {
                        tx.mark_changed(*node, *aspect)?;
                    } else {
                        tx.mark_changed_with_regions(*node, *aspect, changed_regions)?;
                    }
                }
            }
            SetChange::DenseGridRgba {
                family_id,
                rgba,
                aspects,
            } => {
                let family = dense_grids.get(family_id).ok_or_else(|| {
                    SignalError::invalid_input(format!("unknown dense grid family `{family_id}`"))
                })?;
                for index in 0..family.ids.len() {
                    let offset = index * 4;
                    let source = locked.sources.get_mut(&family.ids[index]).ok_or_else(|| {
                        SignalError::invalid_input(format!(
                            "unknown dense source `{}`",
                            family.ids[index]
                        ))
                    })?;
                    set_rgba_signal_value(
                        &mut source.value,
                        rgba[offset],
                        rgba[offset + 1],
                        rgba[offset + 2],
                        rgba[offset + 3],
                    );
                    source.version = super::super::aspects::bump_aspects(source.version, aspects);
                    for aspect in aspects {
                        tx.mark_changed(family.nodes[index], *aspect)?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub(in crate::runtime::core) fn apply_pending_dependency_patches_in_transaction(
    tx: &mut SignalTransaction<'_, (), (), (), SharedStore, ()>,
    store: &SharedStore,
) -> Result<(Vec<PendingCallbackDependencyPatch>, u64), SignalError> {
    let (pending, runtime_read_breadth) = {
        let mut locked = store
            .lock()
            .map_err(|_| SignalError::internal("runtime store mutex poisoned"))?;
        let pending = locked
            .pending_callback_dependency_patches
            .drain(..)
            .collect::<Vec<_>>();
        let runtime_read_breadth = locked.pending_callback_runtime_read_breadth;
        locked.pending_callback_runtime_read_breadth = 0;
        (pending, runtime_read_breadth)
    };
    // Evaluated topology, not a structural replacement: the node keeps the
    // value this transaction computed (see `apply_pending_callback_dependency_patches`).
    for patch in &pending {
        tx.set_evaluated_dependencies(patch.node, patch.dependencies.clone())?;
    }
    Ok((pending, runtime_read_breadth))
}
