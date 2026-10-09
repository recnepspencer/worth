use super::ServingPhysicalRuntime;

impl ServingPhysicalRuntime {
    /// Retires one segment or extent generation displaced by a published rewrite.
    /// A live reader blocks retirement until its protected root is released.
    pub fn retire_displaced_segment(
        &self,
    ) -> Result<(), crate::physical_runtime::PhysicalRetirementDenial> {
        self.retire_displaced_artifact().map(|_| ())
    }

    pub(in crate::physical_runtime) fn completed_displaced_exact(
        &self,
        expected: crate::physical_runtime::durability::DisplacedArtifact,
    ) -> Option<crate::physical_runtime::durability::DisplacedArtifact> {
        self.parts.publication.completed_displaced_exact(expected)
    }

    pub(in crate::physical_runtime) fn retire_displaced_artifact(
        &self,
    ) -> Result<
        Option<crate::physical_runtime::durability::DisplacedArtifact>,
        crate::physical_runtime::PhysicalRetirementDenial,
    > {
        self.parts
            .publication
            .retire_displaced_artifact(&self.checkpoints())
    }
}
