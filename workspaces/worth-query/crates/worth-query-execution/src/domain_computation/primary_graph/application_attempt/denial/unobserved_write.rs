//! Exact attempted write coordinates, without field contents or authority.
use super::{
    AttemptDenialCause, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::EntityId;

/// An existing-entity write missing its exact completed decision observation.
/// These coordinates identify the attempted write; they grant no read or write access.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationUnobservedWrite {
    entity_id: EntityId,
    locator: AspectFieldLocator,
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

    pub(super) fn owned_allocation_capacity_bytes(&self) -> usize {
        self.locator.owned_allocation_capacity_bytes()
    }
}

impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn unobserved_field_write(
        entity_id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Self {
        Self::from_cause(
            WorthQueryApplicationAttemptDenialKind::UnobservedWrite,
            "existing entity field write".to_owned(),
            AttemptDenialCause::UnobservedWrite(Box::new(WorthQueryApplicationUnobservedWrite {
                entity_id,
                locator: locator.clone(),
            })),
        )
    }

    /// Exact coordinates when this refusal is an unobserved existing-entity write.
    pub fn unobserved_write(&self) -> Option<&WorthQueryApplicationUnobservedWrite> {
        match self.cause() {
            Some(AttemptDenialCause::UnobservedWrite(cause)) => Some(cause),
            _ => None,
        }
    }
}
