use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use worth_signal::facade::{
    Aspect, AspectVersion, DependencyEdge, NodeId, SignalRuntime as NativeRuntime,
};

use crate::expression::model::SignalValue;
use crate::recipe::model::{
    KeyedRecipeFamilySpec, KeyedSourceFamilySpec, RecipeFamilyScopeKeyRequirement, RecipeReadSpec,
    RecipeSpec, WasmAspectId,
};
use crate::runtime::compute_callbacks;
use crate::runtime::compute_callbacks::CapturedHostCapabilityRead;
use crate::runtime::summaries::{
    public_callback_read_ids, CallbackDependencyPatchSummary, CallbackFailureSummary,
    RuntimeStoreSnapshot, StoredCallbackRecipeSnapshot, StoredRecipeSnapshot, StoredSourceSnapshot,
};

use super::aspects::{aspect_version_from_summary, aspect_versions_summary};
use super::DEFAULT_ASPECT;

pub(crate) type SharedStore = Arc<Mutex<RuntimeStore>>;
pub(super) type SharedCallbackDiagnostics = Arc<Mutex<BTreeMap<String, CallbackDiagnosticState>>>;
pub type SharedCore = Rc<RefCell<super::RuntimeCore>>;
pub(super) type WasmRuntime = NativeRuntime<(), (), (), SharedStore, ()>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MergePolicyPreviewRequest {
    pub source_branch_id: u64,
    pub target_branch_id: u64,
    #[serde(default)]
    pub conflict_policy_name: Option<String>,
    #[serde(default)]
    pub conflict_isolation_policy_name: Option<String>,
    #[serde(default)]
    pub identity_matcher_name: Option<String>,
    #[serde(default)]
    pub deletion_policy_name: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct CatalogEntry {
    pub(super) node: NodeId,
    pub(super) produced_aspects: Vec<Aspect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSignalKind {
    Input,
    Computed,
    Output,
}

#[derive(Debug, Clone)]
pub(super) struct StoredSource {
    pub(super) value: SignalValue,
    pub(super) version: AspectVersion,
}

#[derive(Debug, Clone)]
pub(super) struct StoredRecipe {
    pub(super) definition: StoredRecipeDefinition,
    pub(super) origin: StoredRecipeOrigin,
    pub(super) value: SignalValue,
    pub(super) version: AspectVersion,
    pub(super) initialized: bool,
    pub(super) output_identity: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StoredRecipeOrigin {
    ExprSpec,
    CallbackSignalTracked,
    CallbackConstantizedNoSignalReads,
}

#[derive(Debug, Clone)]
pub(super) enum StoredRecipeDefinition {
    Expr(RecipeSpec),
    Callback(StoredComputeCallbackRecipe),
}

#[derive(Debug, Clone)]
pub(super) struct StoredComputeCallbackRecipe {
    pub(super) id: String,
    pub(super) token: compute_callbacks::ComputeCallbackToken,
    pub(super) reads: Vec<RecipeReadSpec>,
    pub(super) host_capability_reads: Vec<CapturedHostCapabilityRead>,
    pub(super) produces_aspects: Option<Vec<WasmAspectId>>,
}

impl StoredRecipeDefinition {
    pub(super) fn reads(&self) -> &[RecipeReadSpec] {
        match self {
            Self::Expr(spec) => &spec.reads,
            Self::Callback(recipe) => &recipe.reads,
        }
    }

    pub(super) fn produces_aspects(&self) -> Option<&[WasmAspectId]> {
        match self {
            Self::Expr(spec) => spec.produces_aspects.as_deref(),
            Self::Callback(recipe) => recipe.produces_aspects.as_deref(),
        }
    }

    pub(super) fn exportable_spec(&self) -> Option<&RecipeSpec> {
        match self {
            Self::Expr(spec) => Some(spec),
            Self::Callback(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct StoredSourceFamily {
    pub(super) spec: KeyedSourceFamilySpec,
}

#[derive(Debug, Clone)]
pub(super) struct PendingCallbackDependencyPatch {
    pub(super) node: NodeId,
    pub(super) id: String,
    pub(super) previous_reads: Vec<RecipeReadSpec>,
    pub(super) reads: Vec<RecipeReadSpec>,
    pub(super) host_capability_reads: Vec<CapturedHostCapabilityRead>,
    pub(super) dependencies: Vec<DependencyEdge>,
    pub(super) previous_dependency_count: usize,
    pub(super) runtime_read_breadth: usize,
}

#[derive(Debug, Clone)]
pub(super) struct StoredRecipeFamily {
    pub(super) spec: KeyedRecipeFamilySpec,
    pub(super) scope_key_requirement: RecipeFamilyScopeKeyRequirement,
}

#[derive(Debug, Clone, Default)]
pub(super) struct CallbackDiagnosticState {
    pub(super) current_reads: Vec<String>,
    pub(super) host_capability_reads: Vec<CapturedHostCapabilityRead>,
    pub(super) purity_posture: Option<String>,
    pub(super) last_runtime_read_breadth: u64,
    pub(super) last_dependency_patch: Option<CallbackDependencyPatchSummary>,
    pub(super) last_failure: Option<CallbackFailureSummary>,
}

#[derive(Debug, Clone)]
pub(super) struct DenseGridFamily {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) ids: Vec<String>,
    pub(super) nodes: Vec<NodeId>,
    pub(super) key_to_index: BTreeMap<String, usize>,
    pub(super) produced_aspects: Vec<Aspect>,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct KeyedEnsureStats {
    pub(super) source_hits: usize,
    pub(super) source_created: usize,
    pub(super) recipe_hits: usize,
    pub(super) recipe_created: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct PackedFieldReadStats {
    pub(super) key_reads: usize,
    pub(super) source_reads: usize,
    pub(super) recipe_reads: usize,
    pub(super) recipe_cold_reads: usize,
    pub(super) runtime_read_ms: f64,
    pub(super) field_extract_ms: f64,
    pub(super) fields_packed: usize,
}

#[derive(Debug, Clone)]
pub(super) struct KeyedTarget {
    pub(super) id: String,
    pub(super) node: NodeId,
}

#[derive(Debug, Clone, Default)]
pub(super) struct BranchRuntimeMetadata {
    pub(super) catalog: BTreeMap<String, CatalogEntry>,
    pub(super) nodes_by_id: BTreeMap<NodeId, String>,
    pub(super) dense_grids: BTreeMap<String, Arc<DenseGridFamily>>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct BranchRuntimeState {
    pub(super) metadata: BranchRuntimeMetadata,
    pub(super) store: RuntimeStoreSnapshot,
    pub(super) authored_graph_generation: u64,
}

#[derive(Debug, Clone, Default)]
pub(super) struct WebRuntimeMetrics {
    pub(super) output_serialization_count: u64,
    pub(super) output_serialization_breadth: u64,
    pub(super) compatibility_read_count: u64,
    pub(super) compatibility_read_breadth: u64,
    pub(super) compute_callback_dependency_patch_count: u64,
    pub(super) compute_callback_dependency_patch_added_count: u64,
    pub(super) compute_callback_dependency_patch_removed_count: u64,
    pub(super) compute_callback_dependency_patch_retained_count: u64,
    pub(super) compute_callback_runtime_read_breadth: u64,
    pub(super) compute_callback_constant_no_signal_read_classification_count: u64,
    pub(super) compute_callback_signal_tracked_classification_count: u64,
    pub(super) compute_callback_missing_unavailability_count: u64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeStore {
    pub(super) sources: BTreeMap<String, StoredSource>,
    pub(super) recipes: BTreeMap<String, StoredRecipe>,
    pub(super) source_families: BTreeMap<String, StoredSourceFamily>,
    pub(super) recipe_families: BTreeMap<String, StoredRecipeFamily>,
    pub(super) pending_callback_dependency_patches: Vec<PendingCallbackDependencyPatch>,
    pub(super) pending_callback_runtime_read_breadth: u64,
}

impl RuntimeStore {
    pub(super) fn read_value(&self, id: &str) -> Option<SignalValue> {
        self.sources
            .get(id)
            .map(|source| source.value.clone())
            .or_else(|| self.recipes.get(id).map(|recipe| recipe.value.clone()))
    }

    pub(super) fn snapshot(
        &self,
        catalog: &BTreeMap<String, CatalogEntry>,
    ) -> RuntimeStoreSnapshot {
        RuntimeStoreSnapshot {
            sources: self
                .sources
                .iter()
                .map(|(id, source)| StoredSourceSnapshot {
                    id: id.clone(),
                    value: source.value.clone(),
                    version: source.version.get(DEFAULT_ASPECT),
                    produces_aspects: catalog.get(id).map(|entry| {
                        entry
                            .produced_aspects
                            .iter()
                            .map(|aspect| aspect.id())
                            .collect()
                    }),
                    aspect_versions: aspect_versions_summary(
                        source.version,
                        catalog
                            .get(id)
                            .map(|entry| {
                                entry
                                    .produced_aspects
                                    .iter()
                                    .map(|aspect| aspect.id())
                                    .collect::<Vec<_>>()
                            })
                            .as_deref(),
                    ),
                })
                .collect(),
            recipes: self
                .recipes
                .iter()
                .map(|(id, recipe)| StoredRecipeSnapshot {
                    id: id.clone(),
                    value: recipe.value.clone(),
                    version: recipe.version.get(DEFAULT_ASPECT),
                    produces_aspects: recipe
                        .definition
                        .produces_aspects()
                        .map(|aspects| aspects.to_vec()),
                    aspect_versions: aspect_versions_summary(
                        recipe.version,
                        recipe.definition.produces_aspects(),
                    ),
                    initialized: recipe.initialized,
                    output_identity: recipe.output_identity.clone(),
                    callback: match &recipe.definition {
                        StoredRecipeDefinition::Expr(_) => None,
                        StoredRecipeDefinition::Callback(callback) => {
                            let callback_read_ids = callback
                                .reads
                                .iter()
                                .map(|read| read.id().to_owned())
                                .collect::<Vec<_>>();
                            Some(StoredCallbackRecipeSnapshot {
                                token_slot: callback.token.slot,
                                token_generation: callback.token.generation,
                                reads: public_callback_read_ids(&callback_read_ids),
                                host_capability_reads: callback.host_capability_reads.clone(),
                            })
                        }
                    },
                })
                .collect(),
        }
    }

    pub(super) fn restore_snapshot(&mut self, snapshot: RuntimeStoreSnapshot) {
        self.sources = snapshot
            .sources
            .into_iter()
            .map(|source| {
                (
                    source.id,
                    StoredSource {
                        value: source.value,
                        version: aspect_version_from_summary(
                            source.version,
                            &source.aspect_versions,
                            source.produces_aspects.as_deref(),
                        ),
                    },
                )
            })
            .collect();
        for recipe in snapshot.recipes {
            if let Some(existing) = self.recipes.get_mut(&recipe.id) {
                let produced_aspects = recipe
                    .produces_aspects
                    .as_deref()
                    .or(existing.definition.produces_aspects());
                existing.value = recipe.value;
                existing.version = aspect_version_from_summary(
                    recipe.version,
                    &recipe.aspect_versions,
                    produced_aspects,
                );
                existing.initialized = recipe.initialized;
                existing.output_identity = recipe.output_identity;
                if let (
                    StoredRecipeDefinition::Callback(existing_callback),
                    Some(callback_snapshot),
                ) = (&mut existing.definition, recipe.callback)
                {
                    existing_callback.reads =
                        super::evaluation::canonicalize_callback_reads(callback_snapshot.reads);
                    existing_callback.host_capability_reads =
                        callback_snapshot.host_capability_reads;
                }
            }
        }
        self.pending_callback_dependency_patches.clear();
        self.pending_callback_runtime_read_breadth = 0;
    }
}

pub(super) fn dispose_callback_recipe_token(recipe: &StoredRecipe) {
    if let StoredRecipeDefinition::Callback(callback) = &recipe.definition {
        let _ = compute_callbacks::dispose_compute(callback.token);
    }
}
