use super::{UiValidationAppearanceClass, UiValidationAppearanceFact, ValidationFacts};
use crate::graph::UiGraphNodeIdentity;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity};

/// A sealed persistent version. Cloning this export never walks the owner table.
#[derive(Clone, Debug)]
pub(crate) struct UiValidationAppearanceFactSnapshot {
    pub(super) owner_revision: u64,
    pub(super) facts: ValidationFacts,
}

impl PartialEq for UiValidationAppearanceFactSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.owner_revision == other.owner_revision && self.facts.root_is_shared_with(&other.facts)
    }
}

impl Eq for UiValidationAppearanceFactSnapshot {}

impl UiValidationAppearanceFactSnapshot {
    pub(crate) fn changed_instances(&self, predecessor: &Self) -> Box<[UiMountedInstanceIdentity]> {
        self.facts
            .changed_keys_with_work(&predecessor.facts)
            .0
            .into()
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) fn class_for(
        &self,
        graph_node: UiGraphNodeIdentity,
        mounted_instance: UiMountedInstanceIdentity,
        node_receipt: Option<UiMountedNodeReceiptIdentity>,
    ) -> Option<UiValidationAppearanceClass> {
        let fact = self.fact_for(graph_node, mounted_instance)?;
        Some(if Some(fact.node_receipt) == node_receipt {
            fact.class
        } else {
            UiValidationAppearanceClass::Stale
        })
    }

    pub(crate) fn fact_basis_for(
        &self,
        graph_node: UiGraphNodeIdentity,
        mounted_instance: UiMountedInstanceIdentity,
    ) -> Option<(u64, u64, UiMountedNodeReceiptIdentity)> {
        let fact = self.fact_for(graph_node, mounted_instance)?;
        Some((fact.identity, fact.revision, fact.node_receipt))
    }

    fn fact_for(
        &self,
        graph_node: UiGraphNodeIdentity,
        mounted_instance: UiMountedInstanceIdentity,
    ) -> Option<&UiValidationAppearanceFact> {
        let (owner_node, fact) = self.facts.get(&mounted_instance)?;
        (*owner_node == graph_node).then_some(fact)
    }

    #[cfg(test)]
    pub(crate) fn fact_count(&self) -> usize {
        self.facts.len()
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        graph_node: UiGraphNodeIdentity,
        mounted_instance: UiMountedInstanceIdentity,
        node_receipt: UiMountedNodeReceiptIdentity,
        class: UiValidationAppearanceClass,
    ) -> Self {
        let mut facts = ValidationFacts::default();
        facts.insert(
            mounted_instance,
            (
                graph_node,
                UiValidationAppearanceFact {
                    identity: 1,
                    revision: 1,
                    node_receipt,
                    class,
                },
            ),
        );
        Self {
            owner_revision: 1,
            facts,
        }
    }
}
