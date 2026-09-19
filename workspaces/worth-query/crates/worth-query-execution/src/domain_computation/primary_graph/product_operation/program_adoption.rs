//! Branch-local application-program adoption.
//!
//! Preparation reads one exact product occurrence, computes the installed
//! source-to-target requirements, selects the bounded existing-state scope,
//! and reserves one Relational-only World publication. Publication accepts
//! only that move-only product; a revision by itself carries no effect power.
//!
//! Phase 1 holds the branch activation gate through candidate preparation,
//! then relies on the prepared World's exact product-head fence during the
//! move-only publication. Durable same-key replay belongs to the later
//! custody progression; this slice provides a fail-closed requirements
//! freshness fence and does not claim idempotent replay.

mod custody;
mod preparation;
mod publication;

pub use custody::{
    WorthQueryBranchAdoptionRecovery, WorthQueryBranchAdoptionRecoveryDenial,
    WorthQueryBranchAdoptionRecoveryFailure, WorthQueryBranchAdoptionRecoveryOutcome,
};

pub use preparation::{
    WorthQueryBranchAdoptionActivationDenial, WorthQueryBranchAdoptionPreparationDenial,
    WorthQueryPreparedBranchAdoption,
};
pub use publication::{
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryPerformedBranchAdoption,
    WorthQueryUnpublishedBranchAdoption,
};

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::WorthQuerySelectedProductOperation;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Describes the owner-computed requirements for moving this exact branch
    /// occurrence to `target`. The returned value is evidence, not a permit;
    /// preparation recomputes and compares it.
    pub fn branch_adoption_requirements(
        &self,
        target: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramAdoptionRequirements, WorthQueryBranchAdoptionPreparationDenial>
    {
        preparation::requirements(self, target)
    }

    /// Prepares one branch-local move to `target` against the exact selected
    /// occurrence and the caller-visible requirements admitted beforehand.
    ///
    /// The requirements are recomputed from installed owner truth and must be
    /// exactly equal; presenting an old or foreign report grants nothing.
    pub fn prepare_branch_adoption(
        &self,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        maximum_selection_work: usize,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
        let application = self.application();
        let gate = application
            .product_runtime
            .activations
            .gate(self.product().branch_identity())
            .map_err(|denial| {
                WorthQueryBranchAdoptionPreparationDenial::ProductActivation(denial.into())
            })?;
        gate.publish(|| {
            preparation::prepare(
                self,
                target,
                expected_requirements,
                maximum_selection_work,
                request,
            )
        })
        .map_err(|denial| {
            WorthQueryBranchAdoptionPreparationDenial::ProductActivation(denial.into())
        })?
    }
}
