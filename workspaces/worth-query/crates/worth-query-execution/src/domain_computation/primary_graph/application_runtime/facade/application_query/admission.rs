//! The runtime's consumer methods over `admission` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn resolve_application_query_limits(
        &self,
        limits: worth_query_installation::facade::WorthQueryInstalledApplicationQueryLimits,
    ) -> worth_query_installation::facade::WorthQueryResolvedApplicationQueryLimits {
        <Self as crate::domain_computation::primary_graph::application_query::WorkLimitOps<
            Schema,
        >>::resolve_application_query_limits(self, limits)
    }
}
