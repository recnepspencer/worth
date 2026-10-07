//! Configure contribution members against the admitted schema and program.
use super::super::home_start::AdmittedEntry;
use super::super::WorthQueryApplicationOpenDenial as Denial;
use super::package_admission::AdmittedInstallation;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use crate::domain_computation::primary_graph::application_contribution::{
    PendingConditionalRegistry, PendingProducerRegistry, WorthQueryApplicationContributionTuple,
    WorthQueryConfiguredApplicationContributions,
};
use crate::domain_computation::primary_graph::invariant_installation::WorthQueryInvariantProgramBasis;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramActivationCell;
use crate::domain_computation::primary_graph::{
    handler::PendingMutationHandlerRegistry, WorthQueryApplicationInvariantFactories,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

pub(super) struct ConfiguredInstallation<Schema: ApplicationSchema> {
    pub(super) runtime: WorthQueryExecutionRuntime,
    pub(super) authority: WorthQueryExecutionInstallationAuthority,
    pub(super) installed: WorthQueryInstalledApplicationSchema<Schema>,
    pub(super) entry: AdmittedEntry<Schema>,
    pub(super) activation: WorthQueryProgramActivationCell,
    pub(super) invariants: WorthQueryApplicationInvariantFactories<Schema>,
    pub(super) handlers: PendingMutationHandlerRegistry<Schema>,
    pub(super) producers: PendingProducerRegistry<Schema>,
    pub(super) conditionals: PendingConditionalRegistry<Schema>,
}

pub(super) fn configure<Schema, Contributions>(
    admitted: AdmittedInstallation<Schema>,
    configuration: Contributions::Configuration,
) -> Result<ConfiguredInstallation<Schema>, Denial>
where
    Schema: ApplicationSchema,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let configured = WorthQueryConfiguredApplicationContributions::<Schema>::configure::<
        Contributions,
    >(&admitted.installed, configuration, admitted.contracts)
    .map_err(Denial::Contributions)?;
    let (mut invariants, handlers, producers, conditionals) =
        configured.into_parts().map_err(Denial::Contributions)?;
    let activation = WorthQueryProgramActivationCell::unpublished();
    if let AdmittedEntry::Program(support) = &admitted.entry {
        invariants.select_by_program(WorthQueryInvariantProgramBasis::admitted(
            std::sync::Arc::clone(&support.roster),
            activation.clone(),
        ));
    }
    Ok(ConfiguredInstallation {
        runtime: admitted.runtime,
        authority: admitted.authority,
        installed: admitted.installed,
        entry: admitted.entry,
        activation,
        invariants,
        handlers,
        producers,
        conditionals,
    })
}
