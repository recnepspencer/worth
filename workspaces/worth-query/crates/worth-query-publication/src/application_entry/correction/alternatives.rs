//! Bounded history pages and exact selection under the current request.
use super::*;
pub use worth_query_execution::facade::primary_graph::{
    WorthQueryCorrectionAlternativeRef, WorthQueryCorrectionAlternativesContinuation,
    WorthQuerySelectedCorrectionAlternative,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryCorrectionAlternativesPage, WorthQueryCorrectionDerivationDenial,
};

/// A disclosed page of alternatives and its native continuation or completed resolution.
pub struct WorthQueryApplicationCorrectionAlternatives<'a, 'p, 's, S> {
    request: WorthQueryApplicationCorrections<'a, 'p, 's, S>,
    page: WorthQueryCorrectionAlternativesPage,
}

impl<'a, 'p, 's, S: ApplicationSchema> WorthQueryApplicationCorrections<'a, 'p, 's, S> {
    /// Reads the first bounded page of the source's canonical reversal alternatives.
    pub fn alternatives(
        self,
        source: &WorthQuerySelectedCorrectionSource<'_, S>,
        limits: WorthQueryCorrectionHistoryLimits,
        maximum_entries: NonZeroUsize,
    ) -> Result<
        WorthQueryApplicationCorrectionAlternatives<'a, 'p, 's, S>,
        WorthQueryApplicationCorrectionDenial,
    > {
        self.alternatives_page(source, limits, maximum_entries, None)
    }

    /// Continues the same native history generation with fresh disclosure admission.
    pub fn continue_alternatives(
        self,
        source: &WorthQuerySelectedCorrectionSource<'_, S>,
        limits: WorthQueryCorrectionHistoryLimits,
        maximum_entries: NonZeroUsize,
        continuation: &WorthQueryCorrectionAlternativesContinuation,
    ) -> Result<
        WorthQueryApplicationCorrectionAlternatives<'a, 'p, 's, S>,
        WorthQueryApplicationCorrectionDenial,
    > {
        self.alternatives_page(source, limits, maximum_entries, Some(continuation))
    }

    fn alternatives_page(
        self,
        source: &WorthQuerySelectedCorrectionSource<'_, S>,
        limits: WorthQueryCorrectionHistoryLimits,
        maximum_entries: NonZeroUsize,
        after: Option<&WorthQueryCorrectionAlternativesContinuation>,
    ) -> Result<
        WorthQueryApplicationCorrectionAlternatives<'a, 'p, 's, S>,
        WorthQueryApplicationCorrectionDenial,
    > {
        use WorthQueryApplicationCorrectionDenial as Denial;
        let page =
            self.application
                .with_application_advancement(self.scope, |phase| {
                    let target = self
                        .application
                        .on_branch(self.branch)
                        .select(&phase)
                        .map_err(Denial::ProductSelection)?;
                    target.correction_alternatives(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &phase, self.principal, self.scope, source,
                limits.maximum_ancestors, limits.maximum_disclosure_results, maximum_entries, after,
            ).map_err(Denial::Derivation)
                })
                .map_err(Denial::ExecutionRequest)??;
        Ok(WorthQueryApplicationCorrectionAlternatives {
            request: self,
            page,
        })
    }
}

impl<'a, S: ApplicationSchema> WorthQueryApplicationCorrectionAlternatives<'a, '_, '_, S> {
    /// Canonical historical alternatives represented on this page.
    pub fn entries(&self) -> &[WorthQueryCorrectionAlternativeRef] {
        self.page.entries()
    }

    /// Continue until this is absent before selecting an alternative for preparation.
    pub fn continuation(&self) -> Option<&WorthQueryCorrectionAlternativesContinuation> {
        self.page.continuation()
    }

    /// Resolves an alternative from any page of this lineage after traversal completes.
    /// Selection rechecks disclosure and retains the native source and reversal evidence.
    pub fn select(
        &self,
        alternative: &WorthQueryCorrectionAlternativeRef,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> Result<WorthQuerySelectedCorrectionAlternative<'a, S>, WorthQueryApplicationCorrectionDenial>
    {
        use WorthQueryApplicationCorrectionDenial as Denial;
        let resolution = self.page.resolution().ok_or(Denial::Derivation(
            WorthQueryCorrectionDerivationDenial::AlternativeResolutionIncomplete,
        ))?;
        self.request
            .application
            .with_application_advancement(self.request.scope, |phase| {
                let target = self
                    .request
                    .application
                    .on_branch(self.request.branch)
                    .select(&phase)
                    .map_err(Denial::ProductSelection)?;
                target
                    .select_correction_alternative(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        &phase,
                        self.request.principal,
                        self.request.scope,
                        alternative,
                        resolution,
                        limits.maximum_ancestors,
                        limits.maximum_disclosure_results,
                    )
                    .map_err(Denial::Derivation)
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
