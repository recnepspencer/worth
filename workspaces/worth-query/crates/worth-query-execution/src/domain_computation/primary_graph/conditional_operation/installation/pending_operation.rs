use super::WorthQueryConditionalRuntimeInstallationDenial;

pub(in crate::domain_computation::primary_graph) trait WorthQueryPendingConditionalOperation<Schema>
{
    fn binding_identity(&self) -> &str;

    fn install(
        self: Box<Self>,
        bridge: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
        graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        affinity: &crate::domain_computation::primary_graph::conditional_operation::publication::ConditionalRuntimeAffinity,
        authoritative_commit_cursor: u64,
    ) -> Result<
        Box<
            dyn crate::domain_computation::primary_graph::conditional_operation::lifecycle::WorthQueryInstalledConditionalOperation<
                Schema,
            >,
        >,
        WorthQueryConditionalRuntimeInstallationDenial,
    >;
}
