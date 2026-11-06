use super::AspectContract;
use crate::aspects::masks::{
    AspectMask, DiagnosticMask, MaskAdmissibilityDenial, MutationMask, ProjectionMask,
};

impl AspectContract {
    /// Projection masks cannot be replaced by diagnostic masks.
    ///
    /// ```compile_fail
    /// use worth_foundational::facade::{
    ///     AspectContract, AspectMask, DiagnosticMask,
    /// };
    ///
    /// fn diagnostic_is_not_projection(
    ///     contract: &AspectContract,
    ///     diagnostic: &AspectMask<DiagnosticMask>,
    /// ) {
    ///     contract.admits_projection_mask(diagnostic);
    /// }
    /// ```
    pub fn admits_projection_mask(
        &self,
        mask: &AspectMask<ProjectionMask>,
    ) -> Result<(), MaskAdmissibilityDenial> {
        self.masks
            .admit_paths_for_shape(mask.paths(), self.shape(), MaskModeAdmission::Projection)
    }

    pub fn admits_mutation_mask(
        &self,
        mask: &AspectMask<MutationMask>,
    ) -> Result<(), MaskAdmissibilityDenial> {
        self.masks
            .admit_paths_for_shape(mask.paths(), self.shape(), MaskModeAdmission::Mutation)
    }

    pub fn admits_diagnostic_mask(
        &self,
        mask: &AspectMask<DiagnosticMask>,
    ) -> Result<(), MaskAdmissibilityDenial> {
        self.masks
            .admit_paths_for_shape(mask.paths(), self.shape(), MaskModeAdmission::Diagnostic)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MaskModeAdmission {
    Projection,
    Mutation,
    Diagnostic,
}
