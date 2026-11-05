//! The runtime composes `application_attempt`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost;

impl<Schema: ApplicationSchema> ApplicationAttemptHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_attempt_host(&self) -> &dyn ApplicationAttemptHost<Schema> {
        self
    }
    fn output_demands(&self) -> &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry{
        &self.output_production.demands()
    }
    fn program_support(&self) -> &Option<crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport<Schema>>{
        &self.program_support
    }
    fn external_effect_transport(
        &self,
    ) -> &std::sync::OnceLock<
        std::sync::Arc<
            dyn crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport,
        >,
    > {
        &self.runtime.external_effect_transport
    }
    fn execution(
        &self,
    ) -> &crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime {
        self.runtime.execution()
    }
    fn primary_graph_authority(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority {
        self.runtime.primary_graph_authority()
    }
    fn signing_owners(&self) -> &std::sync::Mutex<std::collections::BTreeMap<worth_query_declaration::facade::application_program::ApplicationProgramRevision, worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner<Schema>>>{
        &self.signing_owners
    }
    fn next_application_mutation_partition(&self) -> &std::sync::atomic::AtomicU32 {
        &self.runtime.next_application_mutation_partition
    }
    fn next_external_dispatch_attempt(&self) -> &std::sync::atomic::AtomicU64 {
        &self.runtime.next_external_dispatch_attempt
    }
}
