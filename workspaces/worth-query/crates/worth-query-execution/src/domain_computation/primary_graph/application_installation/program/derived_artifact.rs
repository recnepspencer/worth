use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{
        ApplicationConnectionShape, ApplicationDerivedArtifactDeclaration, ApplicationFeature,
        ApplicationOutputGraphShape, ApplicationProgramDefinition,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryProgramArtifactPosture;

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationOutputDemand,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryProducerOutputFamily,
};

use super::WorthQueryProgramApplicationRuntime;

type RootConnection<Schema, Root> = <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootDemand<Schema, Root> = <<RootConnection<Schema, Root> as ApplicationConnectionShape<
    Schema,
>>::Binding as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type DiscoveredRootDemand<Schema, Root> = <<RootConnection<Schema, Root> as ApplicationConnectionShape<
    Schema,
>>::Binding as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Demand;

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

    /// Proves that the concrete performed operation is an authored cause of
    /// the root artifact it is about to publish.
    pub fn validate_program_root_artifact_source<Root, Binding>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>,
        <RootConnection<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding:
            WorthQueryApplicationRequiredOutputConnection<Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let artifact = self.validate_root_artifact_demand::<Root>(0, 0)?;
        Self::validate_artifact_source::<RootConnection<Schema, Root>, Binding>(artifact)
    }

    /// Discovered roots retain the same concrete source-to-artifact contract
    /// as direct required roots.
    pub fn validate_program_discovered_root_artifact_source<Root, Binding>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>,
        <RootConnection<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding:
            WorthQueryApplicationDiscoveredOutputConnection<Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let artifact = self.validate_derived_artifact_demand::<
            RootConnection<Schema, Root>,
            DiscoveredRootDemand<Schema, Root>,
        >(0, 0)?;
        Self::validate_artifact_source::<RootConnection<Schema, Root>, Binding>(artifact)
    }

    fn validate_artifact_source<Connection, Binding>(
        artifact: Option<ApplicationDerivedArtifactDeclaration>,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Connection: ApplicationConnectionShape<Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let Some(artifact) = artifact else {
            return Ok(());
        };
        let source_feature = <Connection::SourceFeature as ApplicationFeature<Schema>>::IDENTITY;
        if artifact.dependencies().iter().any(|dependency| {
            dependency.identity() == Binding::IDENTITY || dependency.identity() == source_feature
        }) {
            return Ok(());
        }
        Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ForeignDemand,
            "the performed source operation is not a declared dependency of the root artifact",
        ))
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
