use super::{ParameterContribution, ParameterSchemaBinding};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

impl<Schema: ParameterSchemaBinding> WorthQueryApplicationContribution<Schema>
    for ParameterContribution
{
    type Configuration = Arc<AtomicUsize>;

    fn configure(
        setup_calls: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup_calls.fetch_add(1, Ordering::SeqCst);
        setup.invariant(
            super::PositiveParameterCount::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            super::parameter_invariant::resolve_rule,
        )
    }
}
