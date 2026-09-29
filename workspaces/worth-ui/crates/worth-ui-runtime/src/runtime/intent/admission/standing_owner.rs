use crate::runtime::intent::operability::UiIntentOperabilityStandingFact;
use crate::runtime::persistent_index::{UiPersistentOrdMap, UiPersistentOrdSet};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSurfaceBindingGeneration};

mod reobservation;
mod snapshot;
#[cfg(test)]
mod tests;
pub(crate) use snapshot::UiIntentOperabilityStandingFactSnapshot;

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstanceFacts {
    binding: UiSurfaceBindingGeneration,
    routes: UiPersistentOrdMap<Box<str>, UiIntentOperabilityStandingFact>,
}

#[derive(Clone, Default)]
pub(super) struct UiIntentOperabilityStandingOwner {
    facts: UiPersistentOrdMap<UiMountedInstanceIdentity, InstanceFacts>,
    bindings: UiPersistentOrdMap<
        UiSurfaceBindingGeneration,
        UiPersistentOrdSet<UiMountedInstanceIdentity>,
    >,
    /// The instances holding a fact for each route, so a condition change
    /// visits only the facts of the routes that read it.
    routes: UiPersistentOrdMap<Box<str>, UiPersistentOrdSet<UiMountedInstanceIdentity>>,
    revision: u64,
}

pub(crate) struct UiPreparedIntentOperabilityReceiptSuccession {
    pub(super) predecessor: Option<UiIntentOperabilityStandingFactSnapshot>,
    pub(super) successor: Option<UiIntentOperabilityStandingOwner>,
}

impl UiIntentOperabilityStandingOwner {
    pub(super) fn prepare_receipt_succession(
        &self,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        successor: &crate::mounting::UiMountedNodeReceiptBasis,
    ) -> Self {
        let mut prepared = self.clone();
        for (instance, facts) in self.facts.iter() {
            let Some(successor_receipt) = successor.receipt_for(*instance) else {
                continue;
            };
            let mut rebound = facts.clone();
            for (route, fact) in facts.routes.iter() {
                if mounted
                    .validate_owner_receipt_source(*instance, fact.node_receipt())
                    .is_err()
                {
                    continue;
                }
                let mut rebound_fact = fact.clone();
                rebound_fact.rebind_node_receipt(successor_receipt);
                rebound.routes.insert(route.clone(), rebound_fact);
            }
            prepared.facts.insert(*instance, rebound);
        }
        prepared
    }

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
        let previous = self.facts.get(&instance);
        let previous_binding = previous.map(|previous| previous.binding);
        let dropped: Vec<Box<str>> = previous
            .map(|previous| {
                previous
                    .routes
                    .iter()
                    .filter(|(route, _)| row.routes.get(route.as_ref()).is_none())
                    .map(|(route, _)| route.clone())
                    .collect()
            })
            .unwrap_or_default();
        let added: Vec<Box<str>> = row
            .routes
            .iter()
            .filter(|(route, _)| {
                previous.is_none_or(|previous| previous.routes.get(route.as_ref()).is_none())
            })
            .map(|(route, _)| route.clone())
            .collect();
        for route in &dropped {
            self.remove_route_member(route, instance);
        }
        for route in added {
            let mut members = self.routes.get(&route).cloned().unwrap_or_default();
            members.insert(instance);
            self.routes.insert(route, members);
        }
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
        let routes = route_names(row);
        let revision = self.next_revision();
        self.remove_binding_member(binding, instance);
        for route in &routes {
            self.remove_route_member(route, instance);
        }
        self.facts.remove(&instance);
        self.revision = revision;
    }

    pub(super) fn retire_binding(&mut self, binding: UiSurfaceBindingGeneration) {
        let Some(members) = self.bindings.get(&binding).cloned() else {
            return;
        };
        let revision = self.next_revision();
        for instance in members.iter() {
            if let Some(row) = self.facts.get(instance) {
                for route in &route_names(row) {
                    self.remove_route_member(route, *instance);
                }
            }
            self.facts.remove(instance);
        }
        self.bindings.remove(&binding);
        self.revision = revision;
    }

    pub(super) fn rebind_surface(
        &mut self,
        predecessor: UiSurfaceBindingGeneration,
        successor: UiSurfaceBindingGeneration,
    ) {
        let Some(members) = self.bindings.get(&predecessor).cloned() else {
            return;
        };
        for instance in members.iter() {
            let mut row = self
                .facts
                .get(instance)
                .expect("standing binding membership retains its fact row")
                .clone();
            row.binding = successor;
            let rebound: Vec<_> = row
                .routes
                .iter()
                .map(|(route, fact)| {
                    let mut rebound = fact.clone();
                    rebound.rebind_surface(successor);
                    (route.clone(), rebound)
                })
                .collect();
            for (route, fact) in rebound {
                row.routes.insert(route, fact);
            }
            self.replace_instance(*instance, row);
        }
    }

    pub(super) fn prepare_cleared(&self) -> Self {
        Self {
            facts: Default::default(),
            bindings: Default::default(),
            routes: Default::default(),
            revision: if self.facts.is_empty() {
                self.revision
            } else {
                self.next_revision()
            },
        }
    }

    pub(super) fn clear(&mut self) {
        *self = self.prepare_cleared();
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

    fn remove_route_member(&mut self, route: &str, instance: UiMountedInstanceIdentity) {
        let Some(members) = self.routes.get(route) else {
            return;
        };
        let mut members = members.clone();
        members.remove(&instance);
        if members.is_empty() {
            self.routes.remove(&Box::from(route));
        } else {
            self.routes.insert(route.into(), members);
        }
    }

    fn next_revision(&self) -> u64 {
        self.revision
            .checked_add(1)
            .expect("bounded standing-fact revision exhausted")
    }
}

fn route_names(row: &InstanceFacts) -> Vec<Box<str>> {
    row.routes.iter().map(|(route, _)| route.clone()).collect()
}
