use crate::physical_runtime::PhysicalRootPublicationPreparationFailureCause;

/// This posture covers root preparation only. Even `NotStarted` cannot erase
/// the durable WAL and settled data of the enclosing managed mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootPreparationEffectPosture {
    NotStarted,
    InspectionRequired,
}

/// Diagnostic evidence, not retry authority or a retained publication plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalMutationRootPreparationFailure {
    cause: PhysicalRootPublicationPreparationFailureCause,
    posture: PhysicalRootPreparationEffectPosture,
}

impl PhysicalMutationRootPreparationFailure {
    pub(in crate::physical_runtime) fn new(
        cause: PhysicalRootPublicationPreparationFailureCause,
        posture: PhysicalRootPreparationEffectPosture,
    ) -> Self {
        Self { cause, posture }
    }

    pub fn cause(&self) -> &PhysicalRootPublicationPreparationFailureCause {
        &self.cause
    }

    pub const fn effect_posture(&self) -> PhysicalRootPreparationEffectPosture {
        self.posture
    }
}
#[cfg(test)]
#[path = "root_preparation_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
