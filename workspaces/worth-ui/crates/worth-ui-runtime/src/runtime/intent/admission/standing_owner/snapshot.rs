use super::{InstanceFacts, UiIntentOperabilityStandingFact, UiPersistentOrdMap};
use worth_ui_host_contract::UiMountedInstanceIdentity;

#[derive(Clone, Debug)]
pub(crate) struct UiIntentOperabilityStandingFactSnapshot {
    pub(super) owner_revision: u64,
    pub(super) facts: UiPersistentOrdMap<UiMountedInstanceIdentity, InstanceFacts>,
}

impl PartialEq for UiIntentOperabilityStandingFactSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.owner_revision == other.owner_revision && self.facts.root_is_shared_with(&other.facts)
    }
}
impl Eq for UiIntentOperabilityStandingFactSnapshot {}

impl UiIntentOperabilityStandingFactSnapshot {
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) fn changed_instances(&self, previous: &Self) -> Box<[UiMountedInstanceIdentity]> {
        self.facts.changed_keys_with_work(&previous.facts).0.into()
    }

    pub(crate) fn fact_for(
        &self,
        graph: crate::graph::UiGraphNodeIdentity,
        instance: UiMountedInstanceIdentity,
        route: &str,
    ) -> Option<&UiIntentOperabilityStandingFact> {
        let fact = self.facts.get(&instance)?.routes.get(route)?;
        (fact.graph_node() == graph).then_some(fact)
    }

    #[cfg(test)]
    pub(crate) fn facts(&self) -> Vec<UiIntentOperabilityStandingFact> {
        self.facts
            .iter()
            .flat_map(|(_, row)| row.routes.iter().map(|(_, fact)| fact.clone()))
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn seal(owner_revision: u64, input: Vec<UiIntentOperabilityStandingFact>) -> Self {
        let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let mut facts = UiPersistentOrdMap::default();
        for fact in input {
            let instance = fact.mounted_instance();
            let mut row: InstanceFacts =
                facts
                    .get(&instance)
                    .cloned()
                    .unwrap_or_else(|| InstanceFacts {
                        binding,
                        routes: Default::default(),
                    });
            assert!(
                row.routes.get(fact.route()).is_none(),
                "fixture contains a duplicate standing route"
            );
            row.routes.insert(fact.route().into(), fact);
            facts.insert(instance, row);
        }
        Self {
            owner_revision,
            facts,
        }
    }
}
