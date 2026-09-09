use crate::runtime::intent::operability::UiIntentOperabilityStandingFact;
use crate::runtime::persistent_index::{UiPersistentOrdMap, UiPersistentOrdSet};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSurfaceBindingGeneration};

mod snapshot;
#[cfg(test)]
mod tests;
pub(crate) use snapshot::UiIntentOperabilityStandingFactSnapshot;

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstanceFacts {
    binding: UiSurfaceBindingGeneration,
    routes: UiPersistentOrdMap<Box<str>, UiIntentOperabilityStandingFact>,
}

#[derive(Default)]
pub(super) struct UiIntentOperabilityStandingOwner {
    facts: UiPersistentOrdMap<UiMountedInstanceIdentity, InstanceFacts>,
    bindings: UiPersistentOrdMap<
        UiSurfaceBindingGeneration,
        UiPersistentOrdSet<UiMountedInstanceIdentity>,
    >,
    revision: u64,
}

impl UiIntentOperabilityStandingOwner {
    pub(super) fn record(
        &mut self,
        candidate: &crate::runtime::intent::payload::UiPreparedIntentPayload,
        decision: &crate::runtime::intent::operability::UiIntentOperabilityDecision,
    ) {
        let target = candidate.input_basis().target();
        let instance = target.mounted_instance();
        let route: Box<str> = candidate.declaration_identity().into();
        let previous = self.facts.get(&instance);
        if previous.is_some_and(|row| {
            row.binding == target.binding()
                && row.routes.get(&route).is_some_and(|fact| {
                    fact.graph_node() == candidate.graph_node()
                        && fact.node_receipt() == target.node_receipt()
                        && fact.decision() == decision
                })
        }) {
            return;
        }
        let revision = self.next_revision();
        let mut row = previous
            .filter(|row| row.binding == target.binding())
            .cloned()
            .unwrap_or_else(|| InstanceFacts {
                binding: target.binding(),
                routes: Default::default(),
            });
        row.routes.insert(
            route,
            UiIntentOperabilityStandingFact::seal(candidate, decision.clone(), revision),
        );
        self.replace_instance(instance, row);
        self.revision = revision;
    }

    fn replace_instance(&mut self, instance: UiMountedInstanceIdentity, row: InstanceFacts) {
        let previous_binding = self.facts.get(&instance).map(|previous| previous.binding);
        if previous_binding != Some(row.binding) {
            if let Some(binding) = previous_binding {
                self.remove_binding_member(binding, instance);
            }
            let mut members = self.bindings.get(&row.binding).cloned().unwrap_or_default();
            members.insert(instance);
            self.bindings.insert(row.binding, members);
        }
        self.facts.insert(instance, row);
    }

    pub(super) fn retire_instance(&mut self, instance: UiMountedInstanceIdentity) {
        let Some(row) = self.facts.get(&instance) else {
            return;
        };
        let binding = row.binding;
        let revision = self.next_revision();
        self.remove_binding_member(binding, instance);
        self.facts.remove(&instance);
        self.revision = revision;
    }

    pub(super) fn retire_binding(&mut self, binding: UiSurfaceBindingGeneration) {
        let Some(members) = self.bindings.get(&binding).cloned() else {
            return;
        };
        let revision = self.next_revision();
        for instance in members.iter() {
            self.facts.remove(instance);
        }
        self.bindings.remove(&binding);
        self.revision = revision;
    }

    pub(super) fn clear(&mut self) {
        if self.facts.is_empty() {
            return;
        }
        let revision = self.next_revision();
        self.facts = Default::default();
        self.bindings = Default::default();
        self.revision = revision;
    }

    pub(super) fn snapshot(&self) -> UiIntentOperabilityStandingFactSnapshot {
        UiIntentOperabilityStandingFactSnapshot {
            owner_revision: self.revision,
            facts: self.facts.clone(),
        }
    }

    fn remove_binding_member(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        instance: UiMountedInstanceIdentity,
    ) {
        let mut members = self
            .bindings
            .get(&binding)
            .expect("standing fact has binding membership")
            .clone();
        members.remove_with_work(&instance);
        if members.is_empty() {
            self.bindings.remove(&binding);
        } else {
            self.bindings.insert(binding, members);
        }
    }

    fn next_revision(&self) -> u64 {
        self.revision
            .checked_add(1)
            .expect("bounded standing-fact revision exhausted")
    }
}
