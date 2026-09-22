use std::collections::BTreeMap;
use std::sync::Arc;

use super::plan::CompiledWorkflowSemanticPlan;
use super::publication_binding::WorkflowDefinitionPublicationBinding;

mod counters;
mod key;
pub use counters::WorthQueryWorkflowCompilationReuseCounters;
pub(super) use key::{
    WorkflowDefinitionCompilationReuseDenial, WorkflowDefinitionPublicationReuseKey,
    WorkflowDefinitionSemanticReuseKey,
};

pub(super) const DEFAULT_WORKFLOW_COMPILATION_RETAINED_BYTE_BUDGET: usize = 64 * 1024 * 1024;

struct RetainedSemanticPlan {
    plan: Arc<CompiledWorkflowSemanticPlan>,
    retained_bytes: usize,
}

struct RetainedPublicationBinding {
    semantic_key: WorkflowDefinitionSemanticReuseKey,
    binding: WorkflowDefinitionPublicationBinding,
    retained_bytes: usize,
    last_use: u64,
}

pub(super) struct ReusedWorkflowDefinitionCompilation {
    pub(super) semantic: Arc<CompiledWorkflowSemanticPlan>,
    pub(super) binding: WorkflowDefinitionPublicationBinding,
}

pub(in crate::domain_computation::primary_graph) struct WorkflowDefinitionCompilationReuse {
    maximum_retained_bytes: usize,
    retained_bytes: usize,
    use_sequence: u64,
    semantics: BTreeMap<WorkflowDefinitionSemanticReuseKey, RetainedSemanticPlan>,
    publications: BTreeMap<WorkflowDefinitionPublicationReuseKey, RetainedPublicationBinding>,
    warm_hits: usize,
    cold_misses: usize,
    cold_retains: usize,
    semantic_reuse_hits: usize,
    evictions: usize,
    denials: usize,
    peak_retained_bytes: usize,
}

impl WorkflowDefinitionCompilationReuse {
    pub(super) fn new(maximum_retained_bytes: usize) -> Self {
        Self {
            maximum_retained_bytes,
            retained_bytes: 0,
            use_sequence: 0,
            semantics: BTreeMap::new(),
            publications: BTreeMap::new(),
            warm_hits: 0,
            cold_misses: 0,
            cold_retains: 0,
            semantic_reuse_hits: 0,
            evictions: 0,
            denials: 0,
            peak_retained_bytes: 0,
        }
    }

    pub(super) fn reuse(
        &mut self,
        publication_key: WorkflowDefinitionPublicationReuseKey,
        semantic_key: &WorkflowDefinitionSemanticReuseKey,
    ) -> Option<ReusedWorkflowDefinitionCompilation> {
        let Some(retained) = self.publications.get_mut(&publication_key) else {
            self.cold_misses = self.cold_misses.saturating_add(1);
            return None;
        };
        if &retained.semantic_key != semantic_key {
            self.cold_misses = self.cold_misses.saturating_add(1);
            return None;
        }
        let Some(semantic) = self.semantics.get(semantic_key) else {
            self.cold_misses = self.cold_misses.saturating_add(1);
            return None;
        };
        let semantic = Arc::clone(&semantic.plan);
        let use_sequence = self.next_use_sequence();
        let retained = self
            .publications
            .get_mut(&publication_key)
            .expect("retained workflow publication remains present");
        retained.last_use = use_sequence;
        self.warm_hits = self.warm_hits.saturating_add(1);
        Some(ReusedWorkflowDefinitionCompilation {
            semantic,
            binding: retained.binding.clone(),
        })
    }

    pub(super) fn retain(
        &mut self,
        publication_key: WorkflowDefinitionPublicationReuseKey,
        semantic_key: WorkflowDefinitionSemanticReuseKey,
        candidate: Arc<CompiledWorkflowSemanticPlan>,
        binding: WorkflowDefinitionPublicationBinding,
    ) -> Result<Arc<CompiledWorkflowSemanticPlan>, WorkflowDefinitionCompilationReuseDenial> {
        if let Some(retained) = self.semantics.get(&semantic_key) {
            if retained.plan.as_ref() != candidate.as_ref() {
                self.denials = self.denials.saturating_add(1);
                return Err(WorkflowDefinitionCompilationReuseDenial::SemanticCollision);
            }
        }
        let semantic_was_retained = self.semantics.contains_key(&semantic_key);
        let semantic_bytes = if semantic_was_retained {
            0
        } else {
            semantic_key
                .retained_bytes()
                .saturating_add(candidate.retained_bytes)
        };
        let binding_bytes = std::mem::size_of::<WorkflowDefinitionPublicationReuseKey>()
            .saturating_add(semantic_key.retained_bytes())
            .saturating_add(binding.retained_bytes());
        let required = semantic_bytes.saturating_add(binding_bytes);
        if required > self.maximum_retained_bytes {
            self.denials = self.denials.saturating_add(1);
            return Err(WorkflowDefinitionCompilationReuseDenial::ByteBudgetExceeded);
        }
        if let Some(displaced) = self.publications.remove(&publication_key) {
            self.retained_bytes = self.retained_bytes.saturating_sub(displaced.retained_bytes);
            if displaced.semantic_key != semantic_key {
                self.release_unreferenced_semantic(&displaced.semantic_key);
            }
        }
        self.evict_until_available(required, Some(publication_key), Some(&semantic_key));
        if self.retained_bytes.saturating_add(required) > self.maximum_retained_bytes {
            self.release_unreferenced_semantic(&semantic_key);
            self.denials = self.denials.saturating_add(1);
            return Err(WorkflowDefinitionCompilationReuseDenial::ByteBudgetExceeded);
        }
        let semantic = Arc::clone(
            &self
                .semantics
                .entry(semantic_key.clone())
                .or_insert_with(|| RetainedSemanticPlan {
                    plan: candidate,
                    retained_bytes: semantic_bytes,
                })
                .plan,
        );
        if semantic_bytes > 0 {
            self.retained_bytes = self.retained_bytes.saturating_add(semantic_bytes);
        }
        let last_use = self.next_use_sequence();
        self.publications.insert(
            publication_key,
            RetainedPublicationBinding {
                semantic_key,
                binding,
                retained_bytes: binding_bytes,
                last_use,
            },
        );
        self.retained_bytes = self.retained_bytes.saturating_add(binding_bytes);
        self.cold_retains = self.cold_retains.saturating_add(1);
        if semantic_was_retained {
            self.semantic_reuse_hits = self.semantic_reuse_hits.saturating_add(1);
        }
        self.peak_retained_bytes = self.peak_retained_bytes.max(self.retained_bytes);
        Ok(semantic)
    }

    fn evict_until_available(
        &mut self,
        required: usize,
        protected: Option<WorkflowDefinitionPublicationReuseKey>,
        protected_semantic: Option<&WorkflowDefinitionSemanticReuseKey>,
    ) {
        while self.retained_bytes.saturating_add(required) > self.maximum_retained_bytes {
            let Some(key) = self
                .publications
                .iter()
                .filter(|(key, _)| Some(**key) != protected)
                .min_by_key(|(_, retained)| retained.last_use)
                .map(|(key, _)| *key)
            else {
                break;
            };
            let removed = self
                .publications
                .remove(&key)
                .expect("selected retained publication remains present");
            self.retained_bytes = self.retained_bytes.saturating_sub(removed.retained_bytes);
            self.evictions = self.evictions.saturating_add(1);
            if protected_semantic != Some(&removed.semantic_key) {
                self.release_unreferenced_semantic(&removed.semantic_key);
            }
        }
    }

    fn release_unreferenced_semantic(&mut self, key: &WorkflowDefinitionSemanticReuseKey) {
        if self
            .publications
            .values()
            .any(|publication| &publication.semantic_key == key)
        {
            return;
        }
        if let Some(removed) = self.semantics.remove(key) {
            self.retained_bytes = self.retained_bytes.saturating_sub(removed.retained_bytes);
        }
    }

    fn next_use_sequence(&mut self) -> u64 {
        self.use_sequence = self.use_sequence.saturating_add(1);
        self.use_sequence
    }

    pub(in crate::domain_computation::primary_graph) const fn counters(
        &self,
    ) -> WorthQueryWorkflowCompilationReuseCounters {
        WorthQueryWorkflowCompilationReuseCounters::new(
            self.warm_hits,
            self.cold_misses,
            self.cold_retains,
            self.semantic_reuse_hits,
            self.evictions,
            self.denials,
            self.retained_bytes,
            self.peak_retained_bytes,
            self.maximum_retained_bytes,
        )
    }
}

#[cfg(test)]
#[path = "reuse/tests.rs"]
mod tests;
