//! Provider configuration belongs to the declaring contribution's real setup.
use super::super::super::application_resource::WorthQueryApplicationResourceProvider;
use super::*;
use std::sync::Arc;
use worth_query_declaration::facade::application_resource::ApplicationResourceBinding;
use worth_query_installation::facade::{WorthQueryNamedClock, WorthQueryNamedClockSource};

impl<Schema: ApplicationSchema> WorthQueryApplicationContributionSetup<'_, Schema> {
    /// Every resource in this runtime's Native cell uses the same actual clock
    /// Arc/type/source/timeline. Providers own their external work and heaps.
    pub fn resource<Binding, Provider, Clock, Source>(
        &mut self,
        provider: Provider,
        source: Arc<Source>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Provider: WorthQueryApplicationResourceProvider<Schema, Binding, Role = Binding::Provider>,
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
    {
        self.resources.register::<Binding, Provider, Clock, Source>(
            self.installed_schema,
            self.contribution.identity().as_str(),
            provider,
            source,
        )
    }
}
