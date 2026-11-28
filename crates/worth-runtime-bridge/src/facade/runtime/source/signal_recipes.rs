use worth_signal::facade::runtime::{
    ChangeBatch, ChangeBatchAdmission, KeyedRecipe, ObservationHandle, ObservationListener,
    ObservationPolicy, Recipe, RecipeInstance,
};
use worth_signal::facade::specialist::IntoEvaluationOutput;
use worth_signal::facade::{Aspect, TransactionResult};
use worth_signal::facade::{
    ChangedRegion, EvaluationContext, NodeId, SignalError, TemporalPreviousValueReference,
};

use crate::facade::{BridgeMappingId, RuntimeBridge};

pub struct BridgeSignalRecipeObservation {
    runtime_key: u64,
    handle: ObservationHandle,
}

pub struct BridgeSignalRecipe<F> {
    runtime_key: u64,
    pub(super) recipe: RecipeInstance<(), F>,
}

pub struct BridgeSignalRecipeBinding<'a, F> {
    pub(super) owner: &'a BridgeSignalRecipe<F>,
    pub(super) key: KeyedRecipe,
    node: NodeId,
}

impl<F> BridgeSignalRecipeBinding<'_, F> {
    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn recipe(&self) -> &KeyedRecipe {
        &self.key
    }

    pub(super) fn require_owner(&self, runtime: &RuntimeBridge) -> Result<(), SignalError> {
        self.owner.require_owner(runtime)
    }
}

impl<F> BridgeSignalRecipe<F> {
    fn require_owner(&self, runtime: &RuntimeBridge) -> Result<(), SignalError> {
        if self.runtime_key == runtime.signal_runtime_key {
            Ok(())
        } else {
            Err(SignalError::invalid_input(
                "Signal recipe belongs to a different Bridge runtime",
            ))
        }
    }
}

impl RuntimeBridge {
    /// Installs a recipe under a registered mapping's early-cutoff policy.
    pub fn register_mapped_signal_recipe<F>(
        &self,
        mapping: &BridgeMappingId,
        recipe: Recipe<(), F>,
    ) -> Result<BridgeSignalRecipe<F>, SignalError> {
        let registration = self
            .mapping_registry
            .registrations()
            .iter()
            .find(|registration| registration.mapping_id() == mapping)
            .ok_or_else(|| SignalError::invalid_input("Signal recipe mapping is not registered"))?;
        let recipe = match registration.version_comparator() {
            Some(policy) => recipe.with_comparator(policy.clone()),
            None => recipe,
        };
        self.register_signal_recipe(recipe)
    }

    /// Registers a standalone recipe with the original thread-affine Signal owner.
    pub fn register_signal_recipe<F>(
        &self,
        recipe: Recipe<(), F>,
    ) -> Result<BridgeSignalRecipe<F>, SignalError> {
        let recipe = self.with_signal_source(|runtime| runtime.define(recipe))?;
        Ok(BridgeSignalRecipe {
            runtime_key: self.signal_runtime_key,
            recipe,
        })
    }

    /// Resolves a declared family and key through Signal, retaining their exact Bridge owner.
    pub fn bind_signal_recipe<'a, F>(
        &self,
        recipe: &'a BridgeSignalRecipe<F>,
        key: KeyedRecipe,
    ) -> Result<BridgeSignalRecipeBinding<'a, F>, SignalError> {
        recipe.require_owner(self)?;
        if &key.family != recipe.recipe.family() {
            return Err(SignalError::invalid_input(
                "keyed recipe belongs to a different recipe family",
            ));
        }
        let node = self
            .with_signal_source(|runtime| Ok(recipe.recipe.keyed(key.key.clone()).node(runtime)))?;
        Ok(BridgeSignalRecipeBinding {
            owner: recipe,
            key,
            node,
        })
    }

    pub fn observe_signal_recipe<F>(
        &self,
        binding: &BridgeSignalRecipeBinding<'_, F>,
        policy: ObservationPolicy,
        listener: Box<dyn ObservationListener<(), (), (), (), ()>>,
    ) -> Result<BridgeSignalRecipeObservation, SignalError> {
        binding.require_owner(self)?;
        let handle = self.with_signal_source(|runtime| {
            Ok(runtime.observe_nodes(policy, [binding.node], listener))
        })?;
        Ok(BridgeSignalRecipeObservation {
            runtime_key: self.signal_runtime_key,
            handle,
        })
    }

    pub fn unobserve_signal_recipe(
        &self,
        observation: &BridgeSignalRecipeObservation,
    ) -> Result<bool, SignalError> {
        if observation.runtime_key != self.signal_runtime_key {
            return Err(SignalError::invalid_input(
                "Signal observation belongs to a different Bridge runtime",
            ));
        }
        self.with_signal_source(|runtime| Ok(runtime.unobserve(observation.handle)))
    }

    /// Invalidates a registered recipe through Signal's region-aware frontier.
    pub fn invalidate_signal_recipe<F>(
        &self,
        binding: &BridgeSignalRecipeBinding<'_, F>,
        aspect: Aspect,
        regions: Vec<ChangedRegion>,
    ) -> Result<ChangeBatchAdmission, SignalError> {
        binding.require_owner(self)?;
        let batch = ChangeBatch::singleton(binding.node, aspect, regions);
        self.with_signal_source(|runtime| {
            worth_signal::facade::runtime::mark_dirty_batch(runtime.graph_mut(), &batch)
        })
    }

    pub fn evaluate_signal_recipe<F, O>(
        &self,
        binding: &BridgeSignalRecipeBinding<'_, F>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<TransactionResult, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, ()>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        binding.require_owner(self)?;
        self.with_signal_source(|runtime| {
            binding
                .owner
                .recipe
                .keyed(binding.key.key.clone())
                .commit_evaluation(runtime, execution, &mut (), binding.key.memo_key.as_ref())
        })
    }

    /// Captures the prior Signal value from a live ready wake immediately before
    /// evaluating the same installed recipe node.
    pub fn evaluate_signal_recipe_with_previous<F, O>(
        &self,
        binding: &BridgeSignalRecipeBinding<'_, F>,
        wake: &super::signal_time::BridgeSignalWake,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        evaluator: impl for<'ctx> Fn(
                &mut EvaluationContext<'ctx, ()>,
                &TemporalPreviousValueReference,
            ) -> Result<O, SignalError>
            + Sync,
    ) -> Result<TransactionResult, SignalError>
    where
        O: IntoEvaluationOutput,
    {
        binding.require_owner(self)?;
        wake.require_owner(self)?;
        self.with_signal_source(|runtime| {
            runtime.commit_evaluation_with_temporal_previous_value(
                execution,
                &mut (),
                binding.node,
                wake.id(),
                evaluator,
            )
        })
    }
}
