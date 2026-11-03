use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use worth_foundational::facade::ApplicationBuildIdentity;
use worth_query_execution::facade::application_installation::in_memory_program;
use worth_query_execution::facade::application_installation::WorthQueryApplicationCheckpoint;
use worth_query_execution::facade::application_installation::WorthQueryInMemoryApplicationLimits;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
worth_query_decl::facade::worth_query_application! { pub OptionalSchema { owner: "worth.query.test.optional-aspects", version: (1, 0), contributions: [Members], } }
worth_query_decl::facade::worth_query_application! { pub RequiredSchema { owner: "worth.query.test.required-aspects", version: (1, 0), contributions: [Members], } }
worth_query_decl::facade::worth_query_application! { pub ReuseSchema { owner: "worth.query.test.optional-aspects-input-reuse", version: (1, 0), contributions: [Members], } }
impl TopologyModelBinding for ReuseSchema {}
impl BirthSchema for ReuseSchema {
    const REQUIRED: bool = false;
    const INPUT_REUSE: bool = true;
}
impl TopologyModelBinding for OptionalSchema {}
impl BirthSchema for OptionalSchema {
    const REQUIRED: bool = false;
}
impl TopologyModelBinding for RequiredSchema {}
impl BirthSchema for RequiredSchema {
    const REQUIRED: bool = true;
}
worth_query_decl::facade::worth_query_application_contribution! {
    pub contribution Members for Schema: BirthSchema {
        identity: "worth.query.test.optional-aspect-members",
        members: |schema| { crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::schema::declare(crate::domain_computation::primary_graph::output_lineage::ownership_oracle::base_model::declare(schema))
            .application_query(planar_query_definition::<Schema>()).application_query_binding::<PlanarReadBinding<Schema>>() }
    }
}
impl<Schema: BirthSchema> WorthQueryApplicationContribution<Schema> for Members {
    type Configuration = bool;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<Initial<Schema>>()?;
        contracts.producer::<Preserve<Schema>>()?;
        contracts.conditional::<crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::readiness::InitialReady<Schema>>()?;
        contracts.conditional::<crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::readiness::KeepReady<Schema>>()?;
        Ok(())
    }
    fn configure(
        incomplete: bool,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.invariant(
            PositivePlanarTurn::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            |resolver| {
                crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_invariant::resolve_rule(
                    resolver,
                    Arc::new(AtomicUsize::new(0)),
                    Arc::new(AtomicUsize::new(crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_invariant::INVARIANT_PROBE_STANDARD)),
                    None,
                )
            },
        )?;
        setup.handler::<Binding<Schema, false>, _>(Handler)?;
        setup.handler::<OrdinaryBinding<Schema>, _>(Handler)?;
        setup.handler::<Binding<Schema, true>, _>(Handler)?;
        setup.producer::<Initial<Schema>>(Provider { incomplete })?;
        setup.producer::<Preserve<Schema>>(Provider { incomplete })?;
        setup.conditional::<crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::readiness::InitialReady<Schema>>(())?;
        setup.conditional::<crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::readiness::KeepReady<Schema>>(())?;
        Ok(())
    }
}
pub struct Feature;
impl<Schema: BirthSchema> ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "optional-aspect-feature";
}
pub struct Program<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: BirthSchema> ApplicationProgramDefinition<Schema> for Program<Schema> {
    type Contributions = (Members,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<Schema, PositivePlanarTurn>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.optional-aspect-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .native_action::<OrdinaryBinding<Schema>>()
            .finish_action()
            .conditional_operation::<Produce>()
            .conditional_operation::<KeepProduce>()
            .finish()]
    }
}
pub type Application<Schema> = WorthQueryProgramApplicationRuntime<Schema, Program<Schema>>;
pub const ANCHOR: &str = "owner-source:a";
pub fn install<Schema: BirthSchema + ApplicationSchemaComposition<Contributions = (Members,)>>(
    incomplete: bool,
) -> Application<Schema> {
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program<Schema>>::begin()
            .with_schema(&Schema::declaration().unwrap())
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        (incomplete,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111")
            .expect("embedded application build identity"),
        limits::<Schema>(),
        |_, graph, installed| {
            let binding = installed
                .principal_binding(ConsumerPrincipalBinding::reference())
                .unwrap();
            graph.bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("owner").unwrap(),
                1_u64,
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::external_identity("alice"),
                worth_query_decl::facade::authentication::WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for (key, x, y) in [
                (ANCHOR, 1, 1),
                ("owner-source:b", 10, 1),
                ("owner-source:c", 1, 10),
            ] {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Body::reference(),
                        WorthQueryApplicationEntityKey::new(key).unwrap(),
                    )
                    .field(BodyKey::reference(), key.to_owned())
                    .field(Length::reference(), PositiveLength::new(11).unwrap())
                    .field(PositionX::reference(), PositiveLength::new(x).unwrap())
                    .field(PositionY::reference(), PositiveLength::new(y).unwrap()),
                )?;
            }
            for (from, to) in [
                (ANCHOR, "owner-source:b"),
                ("owner-source:b", "owner-source:c"),
                ("owner-source:c", ANCHOR),
            ] {
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    PlanarSuccessor::reference(),
                    format!("{from}-to-{to}"),
                    WorthQueryApplicationEntityKey::new(from).unwrap(),
                    WorthQueryApplicationEntityKey::new(to).unwrap(),
                ))?;
            }
            Ok(())
        },
    )
    .unwrap()
}

pub fn limits<Schema: BirthSchema>() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        worth_query_execution::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(),
        Default::default(),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
}
pub fn restore<Schema: BirthSchema + ApplicationSchemaComposition<Contributions = (Members,)>>(
    checkpoint: WorthQueryApplicationCheckpoint,
) -> Application<Schema> {
    restore_with_projection::<Schema>(checkpoint).0
}

pub fn restore_with_projection<
    Schema: BirthSchema + ApplicationSchemaComposition<Contributions = (Members,)>,
>(
    checkpoint: WorthQueryApplicationCheckpoint,
) -> (
    Application<Schema>,
    WorthQueryApplicationInvariantProjectionAuthority<Schema>,
) {
    worth_query_execution::facade::application_installation::in_memory_program_from_checkpoint_with_projection(
        ApplicationProgramAuthoring::<Schema, Program<Schema>>::begin()
            .with_schema(&Schema::declaration().unwrap())
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        (false,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111")
            .expect("embedded application build identity"),
        limits::<Schema>(),
        checkpoint,
    )
    .unwrap()
}
