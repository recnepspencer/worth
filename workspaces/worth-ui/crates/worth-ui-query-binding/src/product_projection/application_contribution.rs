use worth_query_decl::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

use crate::declaration::{
    WorthUiApplicationSchema, WorthUiRecordContribution, WorthUiStatusActionBinding,
    WorthUiStatusActionHandler, WorthUiStatusIntegrity, WorthUiStatusUpdateBinding,
    WorthUiStatusUpdateHandler,
};

impl WorthQueryApplicationContribution<WorthUiApplicationSchema> for WorthUiRecordContribution {
    type Configuration = ();

    fn configure(
        (): Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, WorthUiApplicationSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.invariant(
            WorthUiStatusIntegrity::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            super::status_integrity::resolve_rule,
        )?;
        setup.handler::<WorthUiStatusUpdateBinding, _>(WorthUiStatusUpdateHandler)?;
        setup.handler::<WorthUiStatusActionBinding, _>(WorthUiStatusActionHandler)
    }
}
