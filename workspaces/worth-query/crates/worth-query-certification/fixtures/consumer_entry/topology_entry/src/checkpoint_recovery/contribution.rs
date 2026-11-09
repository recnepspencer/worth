//! Checkpoint topology composition carries its installed correction dispatcher.
use super::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::application_contribution::*;

/// The checkpoint schema keeps the topology's metadata and its correction dispatcher together.
pub(super) struct CheckpointContribution;
impl ApplicationSchemaContribution<CheckpointSchema> for CheckpointContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::IDENTITY;

    type Members =
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::Members;
}

impl WorthQueryApplicationContribution<CheckpointSchema> for CheckpointContribution {
    type Configuration = TopologyConfiguration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        <TopologyContribution as WorthQueryApplicationContribution<CheckpointSchema>>::contracts(
            contracts,
        )
    }
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        <TopologyContribution as WorthQueryApplicationContribution<CheckpointSchema>>::configure(
            configuration,
            setup,
        )
    }
}
