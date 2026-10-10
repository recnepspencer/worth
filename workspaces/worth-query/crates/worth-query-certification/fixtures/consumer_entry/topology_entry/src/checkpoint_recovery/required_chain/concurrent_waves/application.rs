//! The actual planar producer reads on the owner and returns a managed compute plan.
use super::*;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::application_contribution::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler, WorthQueryDecisionPlan,
};
type Installed = WorthQueryInstalledManagedComputation<
    CheckpointSchema,
    PlanarOutputFeature,
    computation::Computation,
    computation::Owner,
>;
pub(super) struct Handler(Installed);
impl OperationHandler<CheckpointSchema, PlanarMutationBinding<CheckpointSchema>> for Handler {
    fn decide(
        &self,
        input: &PlanarMutation,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            CheckpointSchema,
            PlanarMutationBinding<CheckpointSchema>,
        >,
    ) -> HandlerResult<(), PlanarMutationDenial> {
        <PlanarHandler as OperationHandler<
            CheckpointSchema,
            PlanarMutationBinding<CheckpointSchema>,
        >>::decide(&PlanarHandler, input, reader)
    }
    fn prepare_decision(
        &self,
        input: &PlanarMutation,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            CheckpointSchema,
            PlanarMutationBinding<CheckpointSchema>,
        >,
    ) -> WorthQueryDecisionPlan<(), PlanarMutationDenial> {
        match <PlanarHandler as OperationHandler<
            CheckpointSchema,
            PlanarMutationBinding<CheckpointSchema>,
        >>::decide(&PlanarHandler, input, reader)
        {
            HandlerResult::Completed(()) => {}
            result => return WorthQueryDecisionPlan::immediate(result),
        }
        match self.0.prepare(&input.scope_key) {
            Ok(prepared) => prepared.into_decision_plan(512, |computed| {
                match computed.and_then(|computed| computed.complete()) {
                    Ok(value) => {
                        let _declared_units = value.units;
                        HandlerResult::Completed(())
                    }
                    Err(_) => HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
                }
            }),
            Err(_) => WorthQueryDecisionPlan::immediate(HandlerResult::DomainDenied(
                PlanarMutationDenial::MissingCoordinate,
            )),
        }
    }
    fn candidate_requirements(
        &self,
        input: &PlanarMutation,
        decision: &(),
    ) -> ApplicationCandidateRequirements {
        <PlanarHandler as OperationHandler<
            CheckpointSchema,
            PlanarMutationBinding<CheckpointSchema>,
        >>::candidate_requirements(&PlanarHandler, input, decision)
    }
    fn build_candidate(
        &self,
        input: &PlanarMutation,
        decision: (),
        writer: &mut CandidateWriter<'_, CheckpointSchema, PlanarMutationBinding<CheckpointSchema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        <PlanarHandler as OperationHandler<
            CheckpointSchema,
            PlanarMutationBinding<CheckpointSchema>,
        >>::build_candidate(&PlanarHandler, input, decision, writer)
    }
}
pub(super) struct Contribution;
impl ApplicationSchemaContribution<CheckpointSchema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::IDENTITY;
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<CheckpointSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<CheckpointSchema> {
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::register_members(
            builder,
        )
    }
}
impl WorthQueryApplicationContribution<CheckpointSchema> for Contribution {
    type Configuration = TopologyConfiguration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        <TopologyContribution as WorthQueryApplicationContribution<CheckpointSchema>>::contracts(
            contracts,
        )
    }
    fn configure(
        configuration: TopologyConfiguration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        crate::checkpoint_recovery::computation_partition::configure(setup)?;
        let installed = setup
            .computation::<PlanarOutputFeature, computation::Computation, _>(computation::Owner)?;
        let provider = crate::InitialPlanarProvider::new(
            std::sync::Arc::clone(&configuration.producer_authorization_denials),
            std::sync::Arc::clone(&configuration.producer_domain_denial),
        )
        .with_uniform_decimal_key_width();
        TopologyContribution::configure_topology_with_provider(
            configuration,
            setup,
            provider,
            Handler(installed),
        )
    }
}
pub(super) struct Program;
impl ApplicationProgramDefinition<CheckpointSchema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<program::ChainRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("concurrent-wave-proof");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = demand_policy::feature_specs_with_output::<demand_policy::FinalArtifact>(
            ApplicationFeatureSpec::root::<CheckpointSchema, PlanarOutputFeature>()
                .derived_artifact::<computation::Artifact>()
                .managed_computation::<computation::Computation>()
                .mutation::<PlanarEditBinding<CheckpointSchema>>()
                .mutation::<VertexReplacementBinding<CheckpointSchema>>()
                .conditional_operation::<MutatePlanar>()
                .finish(),
        );
        features.push(
            ApplicationFeatureSpec::root::<CheckpointSchema, program::ChainFeature>()
                .provides::<program::ChainOutput>()
                .conditional_operation::<PublishChain>()
                .finish(),
        );
        features
    }
}
