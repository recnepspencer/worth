use worth_signal::facade::specialist::EvaluationOutput;
use worth_signal::facade::{EvaluationContext, SignalError};

use crate::boundary::errors::WorthSignalJsError;
use crate::expression::model::SignalValue;
use crate::runtime::specialist::VersionSummary;
use crate::runtime::summaries::CallbackDependencyPatchSummary;

use super::super::aspects::aspect_versions_summary;
use super::super::debug::{perf_now_ms, wasm_debug};
use super::super::evaluation::evaluate_node;
use super::super::state::{PendingCallbackDependencyPatch, SharedStore};
use super::super::{RuntimeCore, DEFAULT_ASPECT};

impl RuntimeCore {
    pub fn set_runtime_policy(
        &mut self,
        policy: crate::runtime::policy::RuntimePolicySpec,
    ) -> Result<(), WorthSignalJsError> {
        self.runtime
            .set_runtime_policy(policy.clone().into_native()?);
        self.policy = policy;
        Ok(())
    }

    pub fn take_debug_events(&mut self) -> Vec<String> {
        super::super::debug::take_wasm_debug_events()
    }

    pub fn read_value(&mut self, id: &str) -> Result<SignalValue, WorthSignalJsError> {
        let node = self.node_for_id(id)?;
        let should_recompute_recipe = self
            .lock_store()?
            .recipes
            .get(id)
            .map(|recipe| !recipe.initialized)
            .unwrap_or(false);
        if should_recompute_recipe {
            worth_signal::facade::core::mark_dirty(self.runtime.graph_mut(), node, DEFAULT_ASPECT)
                .map_err(WorthSignalJsError::from)?;
        }
        let evaluator = self.evaluator();
        self.runtime
            .read(node, &self.store, &evaluator)
            .map_err(WorthSignalJsError::from)?;
        self.runtime.clear_live_branch_mutation_residue();
        self.apply_pending_callback_dependency_patches()?;
        let store = self.lock_store()?;
        store
            .read_value(id)
            .ok_or_else(|| WorthSignalJsError::invalid_input(format!("unknown signal id `{id}`")))
    }

    pub fn peek_value(&self, id: &str) -> Result<SignalValue, WorthSignalJsError> {
        let store = self.lock_store()?;
        if let Some(recipe) = store.recipes.get(id) {
            if !recipe.initialized {
                return Err(WorthSignalJsError::invalid_input(format!(
                    "signal id `{id}` is not initialized for callback peek reads"
                )));
            }
            return Ok(recipe.value.clone());
        }
        store
            .sources
            .get(id)
            .map(|source| source.value.clone())
            .ok_or_else(|| WorthSignalJsError::invalid_input(format!("unknown signal id `{id}`")))
    }

    pub fn read_values(
        &mut self,
        ids: Vec<String>,
    ) -> Result<Vec<SignalValue>, WorthSignalJsError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut nodes = Vec::with_capacity(ids.len());
        for id in &ids {
            let node = self.node_for_id(id)?;
            let should_recompute_recipe = self
                .lock_store()?
                .recipes
                .get(id)
                .map(|recipe| !recipe.initialized)
                .unwrap_or(false);
            if should_recompute_recipe {
                worth_signal::facade::core::mark_dirty(
                    self.runtime.graph_mut(),
                    node,
                    DEFAULT_ASPECT,
                )
                .map_err(WorthSignalJsError::from)?;
            }
            nodes.push(node);
        }

        let read_started_at = perf_now_ms();
        let evaluator = self.evaluator();
        let _ = self
            .runtime
            .targets(nodes)
            .on_demand()
            .read_many(&self.store, &evaluator)
            .map_err(WorthSignalJsError::from)?;
        self.runtime.clear_live_branch_mutation_residue();
        self.apply_pending_callback_dependency_patches()?;
        wasm_debug(format!(
            "[worth-signals-wasm] read-many ids={} elapsed_ms={:.1}",
            ids.len(),
            perf_now_ms() - read_started_at
        ));

        let store = self.lock_store()?;
        ids.into_iter()
            .map(|id| {
                store.read_value(&id).ok_or_else(|| {
                    WorthSignalJsError::invalid_input(format!("unknown signal id `{id}`"))
                })
            })
            .collect()
    }

    pub fn read_versions(
        &mut self,
        ids: Vec<String>,
    ) -> Result<Vec<VersionSummary>, WorthSignalJsError> {
        let mut versions = Vec::with_capacity(ids.len());
        let evaluator = self.evaluator();
        for id in ids {
            let node = self.node_for_id(&id)?;
            let version = self
                .runtime
                .read(node, &self.store, &evaluator)
                .map_err(WorthSignalJsError::from)?;
            self.runtime.clear_live_branch_mutation_residue();
            self.apply_pending_callback_dependency_patches()?;
            let produced_aspects = self.catalog.get(&id).map(|entry| {
                entry
                    .produced_aspects
                    .iter()
                    .map(|aspect| aspect.id())
                    .collect::<Vec<_>>()
            });
            versions.push(VersionSummary {
                id,
                version: version.get(DEFAULT_ASPECT),
                aspect_versions: aspect_versions_summary(version, produced_aspects.as_deref()),
            });
        }
        Ok(versions)
    }

    pub(in crate::runtime::core) fn evaluator(
        &self,
    ) -> impl for<'ctx> Fn(
        &mut EvaluationContext<'ctx, SharedStore>,
    ) -> Result<EvaluationOutput, SignalError>
           + Sync {
        let store = self.store.clone();
        let callback_diagnostics = self.callback_diagnostics.clone();
        let nodes_by_id = self.nodes_by_id.clone();
        move |view| evaluate_node(view, &store, &callback_diagnostics, &nodes_by_id)
    }

    pub(in crate::runtime::core) fn apply_pending_callback_dependency_patches(
        &mut self,
    ) -> Result<(), WorthSignalJsError> {
        let (pending, runtime_read_breadth) = self.take_pending_callback_dependency_patches()?;
        // The patch is the read set of the evaluation that just produced the
        // node's value: install it as evaluated topology so the node stays
        // clean. A plain `set_dependencies` would demote it to `MaybeStale`
        // and the next read would recompute a value nothing invalidated.
        let mut graph = self.runtime.graph_mut();
        for patch in &pending {
            graph
                .set_evaluated_dependencies(patch.node, patch.dependencies.clone())
                .map_err(WorthSignalJsError::from)?;
        }
        drop(graph);
        self.record_committed_callback_dependency_patches(pending, runtime_read_breadth)
    }

    pub(in crate::runtime::core) fn take_pending_callback_dependency_patches(
        &self,
    ) -> Result<(Vec<PendingCallbackDependencyPatch>, u64), WorthSignalJsError> {
        let mut store = self.lock_store()?;
        let pending = store
            .pending_callback_dependency_patches
            .drain(..)
            .collect::<Vec<_>>();
        let runtime_read_breadth = store.pending_callback_runtime_read_breadth;
        store.pending_callback_runtime_read_breadth = 0;
        Ok((pending, runtime_read_breadth))
    }

    pub(in crate::runtime::core) fn record_committed_callback_dependency_patches(
        &mut self,
        pending: Vec<PendingCallbackDependencyPatch>,
        runtime_read_breadth: u64,
    ) -> Result<(), WorthSignalJsError> {
        self.web_metrics.compute_callback_runtime_read_breadth = self
            .web_metrics
            .compute_callback_runtime_read_breadth
            .saturating_add(runtime_read_breadth);
        if pending.is_empty() {
            return Ok(());
        }
        let mut diagnostic_updates = Vec::with_capacity(pending.len());
        for patch in pending {
            let added = patch
                .reads
                .len()
                .saturating_sub(patch.previous_dependency_count);
            let removed = patch
                .previous_dependency_count
                .saturating_sub(patch.reads.len());
            let retained = patch.reads.len().min(patch.previous_dependency_count);
            let previous_reads = patch
                .previous_reads
                .iter()
                .map(|read| read.id().to_owned())
                .collect::<Vec<_>>();
            let current_reads = patch
                .reads
                .iter()
                .map(|read| read.id().to_owned())
                .collect::<Vec<_>>();
            self.web_metrics.compute_callback_dependency_patch_count = self
                .web_metrics
                .compute_callback_dependency_patch_count
                .saturating_add(1);
            self.web_metrics
                .compute_callback_dependency_patch_added_count = self
                .web_metrics
                .compute_callback_dependency_patch_added_count
                .saturating_add(added as u64);
            self.web_metrics
                .compute_callback_dependency_patch_removed_count = self
                .web_metrics
                .compute_callback_dependency_patch_removed_count
                .saturating_add(removed as u64);
            self.web_metrics
                .compute_callback_dependency_patch_retained_count = self
                .web_metrics
                .compute_callback_dependency_patch_retained_count
                .saturating_add(retained as u64);
            diagnostic_updates.push((
                patch.id.clone(),
                current_reads.clone(),
                patch.host_capability_reads.clone(),
                CallbackDependencyPatchSummary {
                    previous_reads,
                    current_reads,
                    added_count: added as u64,
                    removed_count: removed as u64,
                    retained_count: retained as u64,
                    runtime_read_breadth: patch.runtime_read_breadth as u64,
                },
                patch.runtime_read_breadth as u64,
            ));
            wasm_debug(format!(
                "[worth-signals-wasm] callback-patch id={} added={} removed={} retained={} runtime_reads={}",
                patch.id, added, removed, retained, patch.runtime_read_breadth
            ));
        }
        let mut diagnostics = self.lock_callback_diagnostics()?;
        for (id, current_reads, host_capability_reads, dependency_patch, runtime_read_breadth) in
            diagnostic_updates
        {
            let state = diagnostics.entry(id).or_default();
            state.current_reads = current_reads;
            state.host_capability_reads = host_capability_reads;
            state.last_runtime_read_breadth = runtime_read_breadth;
            state.last_dependency_patch = Some(dependency_patch);
        }
        Ok(())
    }
}
