//! Owner preparation reads an ordinary sibling fact; commit must reject stale inputs.
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
pub(super) struct Handler(
    Installed,
    std::sync::Arc<std::sync::Mutex<std::collections::BTreeMap<String, u64>>>,
);
impl Handler {
    fn new(installed: Installed) -> Self {
        Self(installed, Default::default())
    }
}
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
        // This is an ordinary field read: it declares no producer prerequisite.
        let next = if let Some(key) = computation::ordinary_predecessor(&input.scope_key) {
            let entity = match reader.resolve_entity(BodyKey::reference(), key) {
                Ok(entity) => entity,
                Err(error) => {
                    return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(error))
                }
            };
            match reader.field(&entity, Length::reference()) {
                Ok(Some(value)) => PositiveLength::get(&value) + 1,
                Ok(None) => {
                    return WorthQueryDecisionPlan::immediate(HandlerResult::DomainDenied(
                        PlanarMutationDenial::MissingCoordinate,
                    ))
                }
                Err(error) => {
                    return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(error))
                }
            }
        } else {
            let worth_query_consumer_values::PlanarOperation::PublishDerivedOutput(output) =
                &input.operation
            else {
                panic!("the producer supplies its declared output")
            };
            PositiveLength::get(&output.value)
        };
        let pending = self.1.clone();
        let key = input.scope_key.clone();
        match self.0.prepare(&key) {
            Ok(prepared) => prepared.into_decision_plan(512, move |computed| {
                match computed.and_then(|computed| computed.complete()) {
                    Ok(_) => {
                        pending.lock().unwrap().insert(key, next);
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
        let mut input = input.clone();
        if let Some(value) = self.1.lock().unwrap().remove(&input.scope_key) {
            input.operation = worth_query_consumer_values::PlanarOperation::PublishDerivedOutput(
                worth_query_consumer_values::PlanarDerivedOutput {
                    body_key: input.scope_key.clone(),
                    value: length(value),
                },
            );
        }
        <PlanarHandler as OperationHandler<
            CheckpointSchema,
            PlanarMutationBinding<CheckpointSchema>,
        >>::build_candidate(&PlanarHandler, &input, decision, writer)
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
            Handler::new(installed),
        )
    }
}
pub(super) struct Program;
impl ApplicationProgramDefinition<CheckpointSchema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<program::ChainRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("ordinary-read-wave-proof");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = demand_policy::feature_specs_with_output::<demand_policy::FinalArtifact>(
            ApplicationFeatureSpec::root::<CheckpointSchema, PlanarOutputFeature>()
                .derived_artifact::<computation::Artifact>()
                .managed_computation::<computation::Computation>()
                .native_action::<PlanarEditBinding<CheckpointSchema>>()
                .finish_action()
                .native_action::<VertexReplacementBinding<CheckpointSchema>>()
                .finish_action()
                .conditional_operation::<MutatePlanar>()
                .finish(),
        );
        features[0] = application::source_feature();
        features.push(
            ApplicationFeatureSpec::root::<CheckpointSchema, program::ChainFeature>()
                .provides::<program::ChainOutput>()
                .conditional_operation::<PublishChain>()
                .finish(),
        );
        features
    }
}
