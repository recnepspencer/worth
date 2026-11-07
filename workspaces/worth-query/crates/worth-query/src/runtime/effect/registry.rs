use std::collections::{BTreeMap, BTreeSet};

use super::declaration::{WorthQueryEffectDeclaration, WorthQueryEffectTriggerSource};
use super::delivery::{WorthQueryEffectCounters, WorthQueryEffectDelivery};
use crate::runtime::{WorthQueryDerivedMaterializationTarget, WorthQueryLiveArtifactTarget};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::runtime) struct WorthQueryEffectTarget {
    effect_name: String,
}

impl WorthQueryEffectTarget {
    pub(in crate::runtime) fn new(effect_name: impl Into<String>) -> Self {
        Self {
            effect_name: effect_name.into(),
        }
    }

    pub(in crate::runtime) fn from_declaration(declaration: &WorthQueryEffectDeclaration) -> Self {
        Self::new(declaration.name())
    }

    pub(in crate::runtime) fn from_name(effect_name: &str) -> Self {
        Self::new(effect_name)
    }

    pub(in crate::runtime) fn as_str(&self) -> &str {
        &self.effect_name
    }
}

pub(in crate::runtime) struct WorthQueryEffectRuntime {
    pub(in crate::runtime::effect) declaration: WorthQueryEffectDeclaration,
    pub(in crate::runtime) deliveries: Vec<WorthQueryEffectDelivery>,
    pub(in crate::runtime::effect) counters: WorthQueryEffectCounters,
    pub(in crate::runtime) latest_delivery: Option<WorthQueryEffectDelivery>,
}
impl WorthQueryEffectRuntime {
    pub(in crate::runtime::effect) fn new(declaration: WorthQueryEffectDeclaration) -> Self {
        Self {
            declaration,
            deliveries: Vec::new(),
            counters: WorthQueryEffectCounters::default(),
            latest_delivery: None,
        }
    }

    pub(in crate::runtime) fn name(&self) -> &str {
        self.declaration.name()
    }

    pub(in crate::runtime) fn effect_policy(&self) -> super::super::WorthQueryEffectPolicy {
        self.declaration.effect_policy()
    }

    pub(in crate::runtime) fn pending_write_intent_count(&self) -> usize {
        self.counters.pending_write_intents()
    }

    pub(in crate::runtime::effect) fn record_delivery(
        &mut self,
        delivery: WorthQueryEffectDelivery,
    ) {
        self.latest_delivery = Some(delivery.clone());
        self.deliveries.push(delivery);
    }

    pub(in crate::runtime) fn latest_delivery(&self) -> Option<&WorthQueryEffectDelivery> {
        self.latest_delivery.as_ref()
    }
}
#[derive(Default)]
pub(in crate::runtime) struct WorthQueryEffectIndex {
    live_to_effects: BTreeMap<WorthQueryLiveArtifactTarget, BTreeSet<WorthQueryEffectTarget>>,
    computed_to_effects:
        BTreeMap<WorthQueryDerivedMaterializationTarget, BTreeSet<WorthQueryEffectTarget>>,
}
impl WorthQueryEffectIndex {
    pub(in crate::runtime::effect) fn register(
        &mut self,
        declaration: &WorthQueryEffectDeclaration,
    ) {
        let effect_target = WorthQueryEffectTarget::from_declaration(declaration);
        self.unregister(&effect_target);
        match &declaration.trigger().source {
            WorthQueryEffectTriggerSource::LiveView { view_name } => {
                self.live_to_effects
                    .entry(WorthQueryLiveArtifactTarget::from_view_name(
                        view_name.clone(),
                    ))
                    .or_default()
                    .insert(effect_target);
            }
            WorthQueryEffectTriggerSource::ComputedView { view_name } => {
                self.computed_to_effects
                    .entry(WorthQueryDerivedMaterializationTarget::new(
                        view_name.clone(),
                    ))
                    .or_default()
                    .insert(effect_target);
            }
        }
    }

    pub(in crate::runtime) fn has_live_dependents(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> bool {
        self.live_to_effects
            .get(target)
            .is_some_and(|dependents| !dependents.is_empty())
    }
    fn unregister(&mut self, effect_target: &WorthQueryEffectTarget) {
        remove_from_index(&mut self.live_to_effects, effect_target);
        remove_from_index(&mut self.computed_to_effects, effect_target);
    }
    pub(in crate::runtime::effect) fn live_candidates<'a>(
        &self,
        targets: impl IntoIterator<Item = &'a WorthQueryLiveArtifactTarget>,
    ) -> Vec<WorthQueryEffectTarget> {
        targets
            .into_iter()
            .filter_map(|target| self.live_to_effects.get(target))
            .flatten()
            .cloned()
            .collect()
    }
    pub(in crate::runtime::effect) fn computed_candidates<'a>(
        &self,
        targets: impl IntoIterator<Item = &'a WorthQueryDerivedMaterializationTarget>,
    ) -> Vec<WorthQueryEffectTarget> {
        targets
            .into_iter()
            .filter_map(|target| self.computed_to_effects.get(target))
            .flatten()
            .cloned()
            .collect()
    }
}
fn remove_from_index<T: Ord + Clone>(
    index: &mut BTreeMap<T, BTreeSet<WorthQueryEffectTarget>>,
    effect_target: &WorthQueryEffectTarget,
) {
    let empty_keys: Vec<T> = index
        .iter_mut()
        .filter_map(|(key, values)| {
            values.remove(effect_target);
            values.is_empty().then(|| key.clone())
        })
        .collect();
    for key in empty_keys {
        index.remove(&key);
    }
}

pub(in crate::runtime) fn insert_effect_runtime(
    effects: &mut BTreeMap<WorthQueryEffectTarget, WorthQueryEffectRuntime>,
    effect_index: &mut WorthQueryEffectIndex,
    declaration: WorthQueryEffectDeclaration,
) {
    effect_index.register(&declaration);
    effects.insert(
        WorthQueryEffectTarget::from_declaration(&declaration),
        WorthQueryEffectRuntime::new(declaration),
    );
}
#[cfg(test)]
#[path = "registry_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
