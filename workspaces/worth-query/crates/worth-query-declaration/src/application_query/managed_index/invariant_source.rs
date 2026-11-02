//! An explicit rule requirement, resolved against the real installed index.
use super::{ApplicationManagedIndexColdSource, ErasedApplicationManagedIndexDefinition};

/// Portable definition meaning; neither a public READ nor a Native Source loan.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationInvariantManagedIndexSource {
    definition: ErasedApplicationManagedIndexDefinition,
    cold: ApplicationManagedIndexColdSource,
}

impl ApplicationInvariantManagedIndexSource {
    /// Platform-fixed policy; applications cannot select logical/physical posture.
    pub const fn interpretation(
        &self,
    ) -> worth_foundational::facade::ManagedSourceInterpretationCommitment {
        worth_foundational::facade::ManagedSourceInterpretationCommitment::LogicalMaterializationV1
    }

    pub fn from_untrusted_parts(
        definition: ErasedApplicationManagedIndexDefinition,
        cold: ApplicationManagedIndexColdSource,
    ) -> Self {
        Self { definition, cold }
    }

    pub fn definition(&self) -> &ErasedApplicationManagedIndexDefinition {
        &self.definition
    }

    pub fn index(&self) -> &str {
        &self.definition.parts().name
    }

    pub const fn cold(&self) -> ApplicationManagedIndexColdSource {
        self.cold
    }
}
