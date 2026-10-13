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
                    Err(_) if computation::catches_denial() => HandlerResult::Completed(()),
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

#[test]
fn compiled_examples_parallel_advancement() {
    let _guard = checkpoint_recovery_test_guard();
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            computation::arm(None);
        }
    }
    let _reset = Reset;
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<Program>(
        None,
        profile,
        4096,
        128 * 1024 * 1024,
        profile.limits().source_currentness_work() as u64,
        diamond::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut left = request
        .demand(PlanarOutputDemand::new("diamond-left"))
        .start_in_program::<Program, program::ChainRoot>(&application)
        .unwrap();
    let mut right = request
        .demand(PlanarOutputDemand::new("diamond-right"))
        .start_in_program::<Program, program::ChainRoot>(&application)
        .unwrap();
    let mut join = request
        .demand(ChainDemand("diamond-join".to_owned()))
        .start_dependent_in_program::<Program, program::ChainConnection>(&application)
        .unwrap();
    for demand in [&mut left, &mut right] {
        assert!(matches!(
            demand.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
    }
    assert!(matches!(
        join.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let schedule =
        computation::Schedule::new(["diamond-left".into(), "diamond-right".into()], None, false);
    let watched = std::sync::Arc::downgrade(&schedule);
    *schedule.owner_completion.lock().unwrap() = Some(std::sync::Arc::new(move |_| {
        let watched = watched.upgrade().unwrap();
        assert_eq!(
            watched.state.lock().unwrap().finished,
            [true, true],
            "both independent members compute before either completes on the owner"
        );
    }));
    computation::arm(Some(schedule.clone()));

    // compiled-example: begin
    use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome;

    let request = application.request(&principal, &scope);
    for (key, y, idempotency) in [
        ("diamond-left", 5, 0x9176_8200_u64),
        ("diamond-right", 54, 0x9176_8201_u64),
    ] {
        let selected = request
            .query(PlanarRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        let changed = request
            .mutate(PlanarSourceAdjustment {
                scope_key: key.to_owned(),
                replacement_y: length(y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&idempotency)
            .execute_performed::<Program, program::ChainRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            changed,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ));
    }
    assert!(matches!(
        join.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    for (key, expected) in [("diamond-left", 6), ("diamond-right", 55)] {
        let output = request
            .query(PlanarOutputRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        assert_eq!(output.rows()[0].value, length(expected));
    }
    // compiled-example: end
    assert_eq!(schedule.state.lock().unwrap().runs, [1, 1]);
}
