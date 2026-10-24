//! Descriptive operated-source coordinates resolve only through canonical native history.
use super::*;

/// A changed observation invalidates traversal only within the same branch occurrence.
pub(super) fn require_current_target(
    retained: &WorthQueryProductBranchReadIdentity,
    current: &WorthQueryProductBranchReadIdentity,
) -> Result<(), WorthQueryCorrectionDerivationDenial> {
    use WorthQueryCorrectionDerivationDenial as Denial;
    if retained == current {
        Ok(())
    } else if retained.same_branch_occurrence(current) {
        Err(Denial::History(crate::domain_computation::primary_graph::provider::WorthQueryAftermathCausalityReadDenial::ForeignContinuation))
    } else {
        Err(Denial::ForeignAlternative)
    }
}

impl<'runtime, S: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, S> {
    /// Freshly resolves one canonical reversal from a completed lineage traversal.
    pub fn select_correction_alternative_coordinates(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        page: &WorthQueryCorrectionAlternativesPage,
        coordinates: &WorthQueryCorrectionSourceCoordinates,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        WorthQuerySelectedCorrectionAlternative<'runtime, S>,
        WorthQueryCorrectionDerivationDenial,
    > {
        use WorthQueryCorrectionDerivationDenial as Denial;
        let resolution = page
            .resolution()
            .ok_or(Denial::AlternativeResolutionIncomplete)?;
        require_current_target(
            &resolution.target,
            &WorthQueryProductBranchReadIdentity::from_observation(self.product().observation()),
        )?;
        let source = self
            .select_correction_source_coordinates(
                phase,
                principal,
                request,
                coordinates,
                maximum_history,
                maximum_results,
            )
            .map_err(|cause| Denial::Selection(Box::new(cause)))?;
        let source = super::super::lineage::source_locator(&source);
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        let fact = self
            .application()
            .primary_provider
            .reversal_of_source_at_basis(&execution, self.product().relational_basis(), &source)
            .map_err(Denial::History)?
            .ok_or(Denial::ForeignAlternative)?;
        if fact.root != resolution.root.coordinates() {
            return Err(Denial::ForeignAlternative);
        }
        self.select_correction_alternative(
            access,
            phase,
            principal,
            request,
            &WorthQueryCorrectionAlternativeRef { fact },
            resolution,
            maximum_history,
            maximum_results,
        )
    }
}
