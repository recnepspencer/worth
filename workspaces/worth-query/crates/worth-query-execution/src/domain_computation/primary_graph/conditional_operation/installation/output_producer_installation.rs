use super::{InstalledOutputProducers, WorthQueryConditionalApplicationRuntimeInstallation};

impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn install_output_producers(
        &mut self,
        producers: InstalledOutputProducers<Schema>,
    ) {
        self.output_producers = producers;
    }
}
