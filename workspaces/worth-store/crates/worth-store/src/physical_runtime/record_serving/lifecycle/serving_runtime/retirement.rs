use super::ServingPhysicalRuntime;

impl ServingPhysicalRuntime {
    /// Retires one segment or extent generation displaced by a published rewrite.
    /// A live reader blocks retirement until its protected root is released.
    pub fn retire_displaced_segment(
        &self,
    ) -> Result<(), crate::physical_runtime::PhysicalRetirementDenial> {
        self.retire_displaced_artifact().map(|_| ())
    }

    /// One bounded incremental reclamation pass (`ReclaimBudget`). Each
    /// artifact is admitted against the remaining budget before its claim and
    /// retired through the same verified planner as [`Self::retire_displaced_segment`],
    /// so a pass never retires more than the budget slice and stops at the
    /// first protected, uncovered or unresolved artifact.
    pub fn reclaim_within_budget(
        &self,
        budget: crate::physical_runtime::PhysicalReclaimBudget,
    ) -> crate::physical_runtime::PhysicalReclaimPass {
        use crate::physical_runtime::{PhysicalReclaimPass, PhysicalReclaimPassEnd as End};
        let mut pass = PhysicalReclaimPass {
            retired_artifacts: 0,
            retired_bytes: 0,
            end: End::Drained,
        };
        loop {
            let Some(next) = self.parts.publication.next_reclaim_bytes() else {
                return pass;
            };
            if !budget.admits(&pass, next) {
                pass.end = End::BudgetExhausted;
                return pass;
            }
            match self.retire_displaced_artifact() {
                Ok(retired) => {
                    pass.retired_artifacts += 1;
                    pass.retired_bytes = pass
                        .retired_bytes
                        .saturating_add(retired.map_or(0, |displaced| displaced.bytes));
                }
                Err(crate::physical_runtime::PhysicalRetirementDenial::Absent) => return pass,
                Err(denial) => {
                    pass.end = End::Denied(denial);
                    return pass;
                }
            }
        }
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
