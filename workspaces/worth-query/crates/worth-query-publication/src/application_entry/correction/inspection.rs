//! Observe a reversal's source and target before requesting fresh inverse admission.
use super::*;

/// A bounded observation; preparation independently rechecks its source and current authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCorrectionApplicability {
    ReadyForPreparation,
}

/// A reversal of one selected performed source under a current request and target.
/// Inspection observes eligibility; keyed preparation admits and captures the inverse.
pub struct WorthQueryApplicationCorrection<'application, 'principal, 'scope, 'source, Schema> {
    pub(super) request: WorthQueryApplicationCorrections<'application, 'principal, 'scope, Schema>,
    pub(super) source: &'source WorthQuerySelectedCorrectionSource<'application, Schema>,
}

impl<'application, 'principal, 'scope, Schema: ApplicationSchema>
    WorthQueryApplicationCorrections<'application, 'principal, 'scope, Schema>
{
    pub fn reverse<'source>(
        self,
        source: &'source WorthQuerySelectedCorrectionSource<'application, Schema>,
    ) -> WorthQueryApplicationCorrection<'application, 'principal, 'scope, 'source, Schema> {
        WorthQueryApplicationCorrection {
            request: self,
            source,
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationCorrection<'_, '_, '_, '_, Schema> {
    /// Reports semantic compatibility under the current declared disclosure query.
    /// This does not grant inverse execution or reserve the inspected target head.
    pub fn inspect(
        &self,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> Result<WorthQueryCorrectionApplicability, WorthQueryApplicationCorrectionDenial> {
        use WorthQueryApplicationCorrectionDenial as Denial;
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
                    .inspect_correction_source(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        &phase,
                        self.request.principal,
                        self.request.scope,
                        self.source,
                        limits.maximum_ancestors,
                        limits.maximum_disclosure_results,
                    )
                    .map_err(Denial::Derivation)?;
                Ok(WorthQueryCorrectionApplicability::ReadyForPreparation)
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
