//! Checks that a temporal conditional binding belongs to the installation
//! being built.

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryHostConditionalPredicateProvider,
    WorthQueryInstalledTemporalConditionalOperation, WorthQueryNamedClock,
    WorthQueryNamedClockSource, WorthQueryTemporalIntentProjector,
};

use super::denial::foreign_binding_denial;
use super::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
};

impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn validate_temporal_binding<
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
    >(
        &self,
        binding: &WorthQueryInstalledTemporalConditionalOperation<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            Node,
            Provider,
            Clock,
            Source,
            Query,
            Parameters,
            QueryResult,
            Scope,
            Projector,
        >,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial>
    where
        Provider: WorthQueryHostConditionalPredicateProvider<Node>,
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
        Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
    {
        let index = self.publication.runtime.installed_packages();
        index
            .validate_conditional_application_node(binding.clocked_node().provider().node())
            .map_err(|denial| foreign_binding_denial(denial.subject()))?;
        self.publication
            .installed_schema
            .validate_installed_query(binding.query())
            .map_err(|denial| foreign_binding_denial(denial.subject()))
    }
}
