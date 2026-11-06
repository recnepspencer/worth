//! Exact attempted write coordinates, without field contents or authority.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::unobserved_write::WorthQueryApplicationUnobservedWrite;
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::EntityId;
use {
    super::AttemptDenialCause, super::WorthQueryApplicationAttemptDenial,
    crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind,
};

impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn unobserved_field_write(
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
