//! Moved whole from `domain_computation::primary_graph::application_attempt::denial::unobserved_write`: the items below and every impl they own.
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::EntityId;

/// An existing-entity write missing its exact completed decision observation.
/// These coordinates identify the attempted write; they grant no read or write access.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationUnobservedWrite {
    pub(in crate::domain_computation::primary_graph) entity_id: EntityId,
    pub(in crate::domain_computation::primary_graph) locator: AspectFieldLocator,
}

impl WorthQueryApplicationUnobservedWrite {
    /// The exact entity occurrence, including its generation.
    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// The exact installed field the attempted write targeted.
    pub fn locator(&self) -> &AspectFieldLocator {
        &self.locator
    }

    pub(in crate::domain_computation::primary_graph) fn owned_allocation_capacity_bytes(
        &self,
    ) -> usize {
        self.locator.owned_allocation_capacity_bytes()
    }
}
