//! Checkpoint topology composition carries its installed correction dispatcher.
use super::*;
use worth_query_decl::facade::{application_aftermath::*, application_schema::*};
use worth_query_host::facade::application_contribution::*;

/// The checkpoint schema keeps the topology's metadata and its correction dispatcher together.
pub(super) struct CheckpointContribution;
impl ApplicationSchemaContribution<CheckpointSchema> for CheckpointContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::IDENTITY;

    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<CheckpointSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<CheckpointSchema> {
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::register_members(
            builder,
        )
    }
    fn visit_correction<Visitor: ApplicationCorrectionVisitor<CheckpointSchema>>(
        source_binding: std::any::TypeId,
        visitor: Visitor,
    ) -> Result<Visitor::Output, Visitor> {
        match computation_partition::entry_correction::recorded::visit(source_binding, visitor) {
            Ok(output) => Ok(output),
            Err(visitor) => <TopologyContribution as ApplicationSchemaContribution<
                CheckpointSchema,
            >>::visit_correction(source_binding, visitor),
        }
    }
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
