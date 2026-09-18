use worth_query_declaration::facade::{
    application_program::{
        ApplicationConnectionShape, ApplicationDerivedArtifactDeclaration,
        ApplicationOutputGraphShape, ApplicationProgramDefinition,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryProgramArtifactPosture;

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputDemand, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryProducerOutputFamily,
};

use super::WorthQueryProgramApplicationRuntime;

type RootConnection<Schema, Root> = <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootDemand<Schema, Root> = <<RootConnection<Schema, Root> as ApplicationConnectionShape<
    Schema,
>>::Binding as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub(super) fn validate_root_artifact_demand<Root>(
        &self,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<Option<ApplicationDerivedArtifactDeclaration>, WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>,
        <RootConnection<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding:
            WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        self.validate_derived_artifact_demand::<
            RootConnection<Schema, Root>,
            RootDemand<Schema, Root>,
        >(maximum_work, maximum_retained_bytes)
    }

    pub(super) fn validate_derived_artifact_demand<Connection, Demand>(
        &self,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<Option<ApplicationDerivedArtifactDeclaration>, WorthQueryOutputDemandDenial>
    where
        Connection: ApplicationConnectionShape<Schema>,
        Demand: WorthQueryApplicationOutputDemand<Schema>,
    {
        let connection = Connection::declaration();
        let producer_family =
            <Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::IDENTITY;
        match self.program.artifact_posture(
            connection.target_instance(),
            std::any::TypeId::of::<Connection::TargetFeature>(),
            producer_family,
        ) {
            WorthQueryProgramArtifactPosture::Legacy => Ok(None),
            WorthQueryProgramArtifactPosture::Undeclared => Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "the target feature has no derived artifact for the selected producer family",
            )),
            WorthQueryProgramArtifactPosture::Installed(artifact)
                if maximum_work > artifact.resource_ceiling().maximum_work() =>
            {
                Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    artifact.stopped_outcome(),
                ))
            }
            WorthQueryProgramArtifactPosture::Installed(artifact)
                if maximum_retained_bytes
                    > artifact.resource_ceiling().maximum_retained_bytes() =>
            {
                Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    artifact.stopped_outcome(),
                ))
            }
            WorthQueryProgramArtifactPosture::Installed(artifact) => Ok(Some(artifact)),
        }
    }
}
