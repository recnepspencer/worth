pub trait WorkLimitOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn resolve_application_query_limits(
        &self,
        limits: worth_query_installation::facade::WorthQueryInstalledApplicationQueryLimits,
    ) -> worth_query_installation::facade::WorthQueryResolvedApplicationQueryLimits {
        limits.resolve()
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > WorkLimitOps<Schema> for HostRuntime
{
}
