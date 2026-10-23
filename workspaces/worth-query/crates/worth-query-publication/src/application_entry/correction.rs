//! Recorded correction selection through the ordinary fresh request.
mod inspection;
mod outcome;
mod preparation;
mod publication;
use super::WorthQueryApplicationRequest;
pub use inspection::{WorthQueryApplicationCorrection, WorthQueryCorrectionApplicability};
pub use outcome::{WorthQueryApplicationCorrectionOutcome, WorthQueryCorrectionValue};
pub use preparation::{
    WorthQueryApplicationCorrectionPreparation, WorthQueryApplicationCorrectionRequest,
    WorthQueryKeyedApplicationCorrection, WorthQueryPreparedRequiredCorrection,
    WorthQueryRequiredCorrectionPreparation,
};
pub use publication::WorthQueryPreparedApplicationCorrection;
use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
pub use worth_query_execution::facade::primary_graph::WorthQuerySelectedCorrectionSource;
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryCorrectionSelectionDenial,
        WorthQueryPrimaryGraphApplicationRuntime, WorthQueryProductBranchAdmissionDenial,
    },
    product::WorthQueryProductBranch,
};
use worth_query_installation::facade::ApplicationSchema;

/// Work bounds for exact ancestry lookup and the declared disclosure query.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryCorrectionHistoryLimits {
    maximum_ancestors: Option<NonZeroUsize>,
    maximum_disclosure_results: NonZeroUsize,
}
impl WorthQueryCorrectionHistoryLimits {
    pub const fn new(
        maximum_ancestors: Option<NonZeroUsize>,
        maximum_disclosure_results: NonZeroUsize,
    ) -> Self {
        Self {
            maximum_ancestors,
            maximum_disclosure_results,
        }
    }
}

#[derive(Debug)]
/// The owning boundary that refused source selection or inverse preparation/publication.
pub enum WorthQueryApplicationCorrectionDenial {
    SourceBindingUnavailable,
    RecordedOutcomeUnavailable,
    Mutation(super::WorthQueryApplicationRequestMutationDenial),
    RequiredOutputs(super::WorthQueryPerformedMutationExecutionDenial),
    ProgramOwner(worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwnerDenial),
    ExecutionRequest(WorthQueryAdvancementDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    Source(WorthQueryCorrectionSelectionDenial),
    Derivation(worth_query_execution::facade::primary_graph::WorthQueryCorrectionDerivationDenial),
}

/// The current principal, request scope and target branch for recorded corrections.
pub struct WorthQueryApplicationCorrections<'application, 'principal, 'scope, Schema> {
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &'scope WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
}

impl<'application, 'principal, 'scope, Schema: ApplicationSchema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
{
    pub fn corrections(
        &self,
    ) -> WorthQueryApplicationCorrections<'application, 'principal, 'scope, Schema> {
        WorthQueryApplicationCorrections {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
        }
    }
}

impl<'application, Schema: ApplicationSchema>
    WorthQueryApplicationCorrections<'application, '_, '_, Schema>
{
    /// Resolves the performed source and reads its declared disclosure query on this target.
    pub fn select(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> Result<
        WorthQuerySelectedCorrectionSource<'application, Schema>,
        WorthQueryApplicationCorrectionDenial,
    > {
        use WorthQueryApplicationCorrectionDenial as Denial;
        self.application
            .with_application_advancement(self.scope, |phase| {
                let target = self
                    .application
                    .on_branch(self.branch)
                    .select(&phase)
                    .map_err(Denial::ProductSelection)?;
                target
                    .select_correction_source(
                        &phase,
                        self.principal,
                        self.scope,
                        receipt,
                        limits.maximum_ancestors,
                        limits.maximum_disclosure_results,
                    )
                    .map_err(Denial::Source)
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
