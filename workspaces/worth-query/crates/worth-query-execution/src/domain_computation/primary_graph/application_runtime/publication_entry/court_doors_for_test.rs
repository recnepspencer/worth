//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn publish_application_runtime_with_ports_for_test(
        self,
        phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<
            '_,
        >,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: SignalConditionalEvaluationBudget,
        source: impl WorthQueryRuntimeTimeSource,
        fault_port: Arc<dyn WorthQueryPrimaryGraphFaultPort>,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        Self::publish_application_runtime_with_ports(
            self,
            phase,
            runtime,
            authority,
            installed_schema,
            conditional_evaluation_budget,
            source,
            fault_port,
        )
    }
}
