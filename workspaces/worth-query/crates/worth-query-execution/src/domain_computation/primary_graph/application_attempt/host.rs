//! What `application_attempt` needs from the runtime that composes it.

/// The runtime's identity plus what `application_attempt` reads or calls above itself.
pub trait ApplicationAttemptHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::authorization::host::AuthorizationHost<Schema>
    + crate::domain_computation::primary_graph::application_output_demand::host::OutputDemandHost<
        Schema,
    > + crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema>
    + crate::domain_computation::primary_graph::handler::host::HandlerHost<Schema>
{
    /// This host as the trait object `application_attempt` is handed.
    fn application_attempt_host(&self) -> &dyn ApplicationAttemptHost<Schema>;
    fn output_demands(&self) -> &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry;
    fn program_support(&self) -> &Option<crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport<Schema>>;
    fn external_effect_transport(
        &self,
    ) -> &std::sync::OnceLock<
        std::sync::Arc<
            dyn crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport,
        >,
    >;
    fn execution(
        &self,
    ) -> &crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime;
    fn primary_graph_authority(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;
    fn signing_owners(&self) -> &std::sync::Mutex<std::collections::BTreeMap<worth_query_declaration::facade::application_program::ApplicationProgramRevision, worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner<Schema>>>;
}
