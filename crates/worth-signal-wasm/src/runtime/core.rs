use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use worth_signal::facade::branch::AdmittedSignalBranchSnapshot;
use worth_signal::facade::history::RuntimeSnapshot;
use worth_signal::facade::runtime::{
    ObservationListener, ObservationNotice, ObservationReadContext,
};
use worth_signal::facade::{DependencyEdge, NodeId, SignalGraph, SignalRuntime as NativeRuntime};

use crate::boundary::errors::WorthSignalJsError;
use crate::recipe::model::RecipeReadSpec;
use crate::runtime::compute_callbacks;
use crate::runtime::diagnostics_callbacks;
use crate::runtime::policy::RuntimePolicySpec;
use crate::runtime::summaries::RuntimeStoreSnapshot;
use crate::runtime::web_callbacks;

mod aspects;
mod branches;
pub(crate) mod certification_digest;
mod debug;
mod diagnostics;
mod envelopes;
mod evaluation;
mod keyed_families;
mod keyed_runtime;
mod merge;
mod merge_caller_restoration;
mod merge_state;
mod runtime_async_lifecycle_certification;
mod signals;
mod snapshots;
mod state;
mod transactions;
mod worker_branch_command_model;
mod worker_branch_commands;
mod worker_branch_reclamation;
mod worker_branch_retirement;
mod worker_branch_snapshot_retirement;
mod worker_callback_definition_publication;
mod worker_callback_reattachment_import;
mod worker_definition_publication_plan;
mod worker_effect_branch_closeout;
mod worker_main_thread_hosted_callbacks;
mod worker_placement_declaration_candidates;

use self::aspects::resolve_selected_aspects;
pub(crate) use self::envelopes::ExactRuntimeRestoreArtifact;
use self::evaluation::canonicalize_callback_reads;
pub(crate) use self::state::SharedStore;
use self::state::{
    dispose_callback_recipe_token, BranchRuntimeState, CallbackDiagnosticState, CatalogEntry,
    DenseGridFamily, RuntimeStore, SharedCallbackDiagnostics, StoredRecipeDefinition, WasmRuntime,
    WebRuntimeMetrics,
};
pub use self::state::{MergePolicyPreviewRequest, SharedCore, WebSignalKind};
pub use self::worker_branch_command_model::{
    WorkerApplyTransactionToBranchReceipt, WorkerApplyTransactionToBranchRequest,
    WorkerBranchBasisReceipt, WorkerBranchRetirementReason, WorkerCloseoutEffectBranchReceipt,
    WorkerCloseoutEffectBranchRequest, WorkerForkBranchReceipt, WorkerForkBranchRequest,
    WorkerRetireBranchReceipt, WorkerRetireBranchRequest, WorkerRetireBranchesReceipt,
    WorkerRetireBranchesRequest,
};
pub(crate) use self::worker_callback_definition_publication::DefinitionEnvelopeCallbackReattachment;
pub(crate) use self::worker_callback_reattachment_import::RuntimeEnvelopeCallbackReattachment;
pub(crate) use self::worker_main_thread_hosted_callbacks::{
    MainThreadHostedCallbackAdmission, MainThreadHostedCallbackClosedInput,
};
use crate::runtime::web_callbacks::ObservationCallbackToken;

const DEFAULT_ASPECT: worth_signal::facade::Aspect = worth_signal::facade::Aspect::new(0);

pub struct RuntimeCore {
    runtime: WasmRuntime,
    store: SharedStore,
    callback_diagnostics: SharedCallbackDiagnostics,
    catalog: BTreeMap<String, CatalogEntry>,
    web_signals: BTreeMap<String, WebSignalKind>,
    nodes_by_id: BTreeMap<NodeId, String>,
    dense_grids: BTreeMap<String, Arc<DenseGridFamily>>,
    branch_states: BTreeMap<u64, BranchRuntimeState>,
    snapshot_states: BTreeMap<(u64, u64), BranchRuntimeState>,
    runtime_snapshots: BTreeMap<(u64, u64), RuntimeSnapshot>,
    admitted_runtime_snapshots: BTreeMap<(u64, u64), AdmittedSignalBranchSnapshot>,
    policy: RuntimePolicySpec,
    web_metrics: WebRuntimeMetrics,
    observation_callback_scope_id: u64,
    diagnostics_callback_scope_id: u64,
}

impl Drop for RuntimeCore {
    fn drop(&mut self) {
        if let Ok(store) = self.store.lock() {
            for recipe in store.recipes.values() {
                dispose_callback_recipe_token(recipe);
            }
        }
        web_callbacks::dispose_runtime_callback_scope(self.observation_callback_scope_id);
        diagnostics_callbacks::dispose_runtime_diagnostics_callback_scope(
            self.diagnostics_callback_scope_id,
        );
    }
}

impl RuntimeCore {
    pub fn new(policy: RuntimePolicySpec) -> Result<Self, WorthSignalJsError> {
        let graph = SignalGraph::new();
        let mut runtime = NativeRuntime::build_for::<SharedStore>(graph);
        runtime.set_runtime_policy(policy.clone().into_native()?);
        let current_branch_id = runtime.current_branch().id.0;
        let mut branch_metadata = BTreeMap::new();
        branch_metadata.insert(current_branch_id, BranchRuntimeState::default());
        Ok(Self {
            runtime,
            store: Arc::new(Mutex::new(RuntimeStore::default())),
            callback_diagnostics: Arc::new(Mutex::new(BTreeMap::new())),
            catalog: BTreeMap::new(),
            web_signals: BTreeMap::new(),
            nodes_by_id: BTreeMap::new(),
            dense_grids: BTreeMap::new(),
            branch_states: branch_metadata,
            snapshot_states: BTreeMap::new(),
            runtime_snapshots: BTreeMap::new(),
            admitted_runtime_snapshots: BTreeMap::new(),
            policy,
            web_metrics: WebRuntimeMetrics::default(),
            observation_callback_scope_id: web_callbacks::allocate_runtime_callback_scope(),
            diagnostics_callback_scope_id:
                diagnostics_callbacks::allocate_runtime_diagnostics_callback_scope(),
        })
    }

    fn lock_store(&self) -> Result<std::sync::MutexGuard<'_, RuntimeStore>, WorthSignalJsError> {
        self.store
            .lock()
            .map_err(|_| WorthSignalJsError::internal("runtime store mutex poisoned"))
    }

    fn lock_callback_diagnostics(
        &self,
    ) -> Result<
        std::sync::MutexGuard<'_, BTreeMap<String, CallbackDiagnosticState>>,
        WorthSignalJsError,
    > {
        self.callback_diagnostics
            .lock()
            .map_err(|_| WorthSignalJsError::internal("callback diagnostics mutex poisoned"))
    }

    fn restore_store(&self, previous: RuntimeStore) -> Result<(), WorthSignalJsError> {
        let mut store = self.lock_store()?;
        *store = previous;
        Ok(())
    }

    fn restore_runtime_store_snapshot(
        &mut self,
        snapshot: RuntimeStoreSnapshot,
    ) -> Result<(), WorthSignalJsError> {
        self.ensure_callback_snapshot_availability(&snapshot)?;
        {
            let mut store = self.lock_store()?;
            store.restore_snapshot(snapshot.clone());
        }
        self.sync_callback_diagnostics_from_store()?;
        self.restore_callback_dependency_shapes(&snapshot)?;
        self.recompute_uninitialized_recipes(&snapshot)?;
        Ok(())
    }

    /// A restored store may carry recipes without a value: a merged branch
    /// state resets every recipe so the merged inputs recompute it, while the
    /// native graph still records those nodes as clean. A read invalidates
    /// only the node it asks for, so an uninitialized recipe upstream of it
    /// would feed `Null` into the evaluation instead of recomputing. When the
    /// store is restored, every uninitialized recipe is invalidated and the
    /// standing demand (published outputs) is evaluated at once, so the branch
    /// is complete before anything observes it and the first read after a
    /// switch, merge, or restore sees the merged truth. Other computeds stay
    /// on-demand and recompute on their next read. O(recipes) once per restore
    /// (the restore already clones every recipe) plus one evaluation of the
    /// demanded nodes, which is the same work a read would have done.
    fn recompute_uninitialized_recipes(
        &mut self,
        snapshot: &RuntimeStoreSnapshot,
    ) -> Result<(), WorthSignalJsError> {
        let mut invalidated = false;
        for recipe in &snapshot.recipes {
            if recipe.initialized {
                continue;
            }
            let Some(node) = self.catalog.get(&recipe.id).map(|entry| entry.node) else {
                continue;
            };
            worth_signal::facade::core::mark_dirty(self.runtime.graph_mut(), node, DEFAULT_ASPECT)
                .map_err(WorthSignalJsError::from)?;
            invalidated = true;
        }
        if !invalidated {
            return Ok(());
        }
        let standing_demand = self.standing_demand_nodes();
        if standing_demand.is_empty() {
            return Ok(());
        }
        let evaluator = self.evaluator();
        let _ = self
            .runtime
            .targets(standing_demand)
            .on_demand()
            .read_many(&self.store, &evaluator)
            .map_err(WorthSignalJsError::from)?;
        self.runtime.clear_live_branch_mutation_residue();
        self.apply_pending_callback_dependency_patches()?;
        Ok(())
    }

    fn node_for_id(&self, id: &str) -> Result<NodeId, WorthSignalJsError> {
        self.catalog
            .get(id)
            .map(|entry| entry.node)
            .ok_or_else(|| WorthSignalJsError::invalid_input(format!("unknown signal id `{id}`")))
    }

    fn ensure_callback_snapshot_availability(
        &mut self,
        snapshot: &RuntimeStoreSnapshot,
    ) -> Result<(), WorthSignalJsError> {
        let store = self.lock_store()?;
        let mut failure: Option<WorthSignalJsError> = None;
        for recipe in &snapshot.recipes {
            let Some(callback_snapshot) = &recipe.callback else {
                continue;
            };
            let Some(existing) = store.recipes.get(&recipe.id) else {
                failure = Some(WorthSignalJsError::callback_failure(
                    "computeCallbackUnavailableForRestore",
                    format!(
                        "snapshot restore requires callback recipe `{}` to exist in the live runtime",
                        recipe.id
                    ),
                    Some(recipe.id.clone()),
                ));
                break;
            };
            let StoredRecipeDefinition::Callback(existing_callback) = &existing.definition else {
                failure = Some(WorthSignalJsError::callback_failure(
                    "computeCallbackUnavailableForRestore",
                    format!(
                        "snapshot restore requires callback recipe `{}` but the live runtime definition is not callback-backed",
                        recipe.id
                    ),
                    Some(recipe.id.clone()),
                ));
                break;
            };
            if existing_callback.token.slot != callback_snapshot.token_slot
                || existing_callback.token.generation != callback_snapshot.token_generation
            {
                failure = Some(WorthSignalJsError::callback_failure(
                    "computeCallbackUnavailableForRestore",
                    format!(
                        "snapshot restore requires callback recipe `{}` generation {}:{} but the live runtime has {}:{}",
                        recipe.id,
                        callback_snapshot.token_slot,
                        callback_snapshot.token_generation,
                        existing_callback.token.slot,
                        existing_callback.token.generation
                    ),
                    Some(recipe.id.clone()),
                ));
                break;
            }
            if !compute_callbacks::is_compute_registered(existing_callback.token) {
                failure = Some(WorthSignalJsError::callback_failure(
                    "computeCallbackUnavailableForRestore",
                    format!(
                        "snapshot restore requires callback recipe `{}` but its callback function is no longer registered",
                        recipe.id
                    ),
                    Some(recipe.id.clone()),
                ));
                break;
            }
        }
        drop(store);
        if let Some(failure) = failure {
            self.web_metrics
                .compute_callback_missing_unavailability_count = self
                .web_metrics
                .compute_callback_missing_unavailability_count
                .saturating_add(1);
            return Err(failure);
        }
        Ok(())
    }

    fn sync_callback_diagnostics_from_store(&self) -> Result<(), WorthSignalJsError> {
        let store = self.lock_store()?;
        let mut diagnostics = self.lock_callback_diagnostics()?;
        diagnostics.retain(|id, _| {
            store
                .recipes
                .get(id)
                .map(|recipe| matches!(recipe.definition, StoredRecipeDefinition::Callback(_)))
                .unwrap_or(false)
        });
        for (id, recipe) in &store.recipes {
            let StoredRecipeDefinition::Callback(callback) = &recipe.definition else {
                continue;
            };
            let state = diagnostics.entry(id.clone()).or_default();
            state.current_reads = callback
                .reads
                .iter()
                .map(|read| read.id().to_owned())
                .collect();
            state.host_capability_reads = callback.host_capability_reads.clone();
            state.last_runtime_read_breadth = 0;
            state.last_dependency_patch = None;
            state.last_failure = None;
        }
        Ok(())
    }

    fn restore_callback_dependency_shapes(
        &mut self,
        snapshot: &RuntimeStoreSnapshot,
    ) -> Result<(), WorthSignalJsError> {
        let mut restored = Vec::new();
        for recipe in &snapshot.recipes {
            let Some(callback_snapshot) = &recipe.callback else {
                continue;
            };
            let node = self.node_for_id(&recipe.id)?;
            let reads = canonicalize_callback_reads(callback_snapshot.reads.clone());
            let dependencies = self.dependencies_for_reads(&reads)?;
            restored.push((node, dependencies));
        }
        let mut graph = self.runtime.graph_mut();
        for (node, dependencies) in restored {
            graph
                .set_dependencies(node, dependencies)
                .map_err(WorthSignalJsError::from)?;
        }
        Ok(())
    }

    fn dependencies_for_reads(
        &self,
        reads: &[RecipeReadSpec],
    ) -> Result<Vec<DependencyEdge>, WorthSignalJsError> {
        let mut dependencies = Vec::new();
        for read in reads {
            let entry = self.catalog.get(read.id()).ok_or_else(|| {
                WorthSignalJsError::invalid_input(format!("unknown read `{}`", read.id()))
            })?;
            let aspects = resolve_selected_aspects(read.aspect_spec())?;
            for aspect in aspects {
                let edge = match read.scope() {
                    Some(scope) => {
                        DependencyEdge::with_partition_scope(entry.node, aspect, scope.clone())
                    }
                    None => DependencyEdge::new(entry.node, aspect),
                };
                dependencies.push(edge);
            }
        }
        Ok(dependencies)
    }
}

pub fn new_shared_core(policy: RuntimePolicySpec) -> Result<SharedCore, WorthSignalJsError> {
    Ok(Rc::new(RefCell::new(RuntimeCore::new(policy)?)))
}

struct WasmWatchListener {
    callback_scope_id: u64,
    callback_token: ObservationCallbackToken,
    signal_id: String,
}

impl ObservationListener<(), (), (), SharedStore, ()> for WasmWatchListener {
    fn on_observation(
        &self,
        ctx: ObservationReadContext<'_, (), (), (), SharedStore, ()>,
        notice: &ObservationNotice<'_>,
    ) {
        web_callbacks::invoke_watch(
            self.callback_scope_id,
            self.callback_token,
            web_callbacks::notice_from_runtime(&self.signal_id, ctx, notice),
        );
    }
}

struct WasmEffectListener {
    callback_scope_id: u64,
    callback_token: ObservationCallbackToken,
}

impl ObservationListener<(), (), (), SharedStore, ()> for WasmEffectListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), SharedStore, ()>,
        _notice: &ObservationNotice<'_>,
    ) {
        web_callbacks::invoke_effect(self.callback_scope_id, self.callback_token);
    }
}
