//! Reuse the installed succession schema and producers with one computation owner.
use super::*;
pub struct NativeContribution;
impl ApplicationSchemaContribution<Schema> for NativeContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <Contribution as ApplicationSchemaContribution<Schema>>::IDENTITY;
    type Members = <Contribution as ApplicationSchemaContribution<Schema>>::Members;
}
impl WorthQueryApplicationContribution<Schema> for NativeContribution {
    type Configuration = Arc<Calls>;
    fn contracts(
        c: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        c.producer::<producers::parent::initial::Producer>()?;
        c.producer::<provider::Producer>()?;
        c.conditional::<producers::parent::initial::Readiness>()?;
        c.conditional::<provider::Readiness>()?;
        producers::child::contracts(c)?;
        producers::independent_parent::contracts(c)?;
        producers::twin::contracts(c)?;
        producers::parameter_routes::contracts(c)?;
        producers::leaf::contracts(c)
    }
    fn configure(
        calls: Arc<Calls>,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<source_control::Binding, _>(source_control::Handler)?;
        setup.handler::<write_fence::Binding, _>(write_fence::Handler)?;
        let selections = Selections::default();
        use producers::parent::{initial, preserve};
        setup.producer::<initial::Producer>(initial::Provider)?;
        setup.producer::<provider::Producer>(provider::Provider)?;
        setup.conditional::<initial::Readiness>(())?;
        setup.conditional::<provider::Readiness>(())?;
        setup.handler::<initial::Binding, _>(initial::Handler(
            selections.clone(),
            Fault::None,
            0,
            false,
        ))?;
        let computation = setup
            .partitioned_computation::<program::SplitParent, model::Computation, _>(
                model::Owner(calls),
            )?;
        setup.handler::<preserve::Binding, _>(handler::Handler {
            original: preserve::Handler(selections.clone(), Fault::None, 0, false),
            computation,
        })?;
        producers::child::install(setup, selections.clone(), Fault::None, 0, false)?;
        producers::independent_parent::install(setup, selections.clone(), Fault::None, 0, false)?;
        producers::twin::install(setup, selections.clone(), Fault::None, 0, false)?;
        producers::parameter_routes::install(setup, selections.clone(), Fault::None, 0, false)?;
        producers::leaf::install(setup, selections, Fault::None, 0, false)
    }
}
pub struct NativeProgram;
impl ApplicationProgramDefinition<Schema> for NativeProgram {
    type Contributions = (NativeContribution,);
    type Outputs = ApplicationProgramOutputs<source_action::Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("succession.walk.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        use producers::{child, independent_parent, leaf, parameter_routes, parent, twin};
        use program::*;
        vec![
            ApplicationFeatureSpec::root::<Schema, SplitEntry>()
                .provides::<SplitEntryOutput>()
                .native_action::<write_fence::Binding>()
                .finish_action()
                .action::<source_control::Binding, source_action::Delivery>()
                .locality_and_change::<model::Locality, source_action::Change>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, SplitParent>()
                .provides::<SplitParentOutput>()
                .conditional_operation::<parent::initial::Operation>()
                .conditional_operation::<parent::preserve::Operation>()
                .conditional_operation::<independent_parent::initial::Operation>()
                .conditional_operation::<independent_parent::preserve::Operation>()
                .derived_artifact::<model::Artifact>()
                .managed_computation::<model::Computation>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, SplitChild>()
                .provides::<SplitChildOutput>()
                .conditional_operation::<child::initial::Operation>()
                .conditional_operation::<child::preserve::Operation>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, SplitLeaf>()
                .conditional_operation::<twin::initial::Operation>()
                .conditional_operation::<twin::preserve::Operation>()
                .conditional_operation::<parameter_routes::initial::Operation>()
                .conditional_operation::<parameter_routes::preserve::Operation>()
                .conditional_operation::<leaf::initial::Operation>()
                .conditional_operation::<leaf::preserve::Operation>()
                .finish(),
        ]
    }
}
pub type App = installation::WorthQueryProgramApplicationRuntime<Schema, NativeProgram>;
pub fn install(calls: Arc<Calls>, worker_only: bool) -> App {
    let schema = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, NativeProgram>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    use worth_query_host::facade::runtime::{
        DeterminismContract, ExecutionPosture, ExecutionRequestPolicy,
    };
    let resources = worth_query_testkit_document_retention::facade::document_world_resources::world_resources(
        256, 4 * 1024 * 1024, 256,
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let resources = if worker_only {
        // The existing public host-selected mode; default accounting is unchanged.
        resources.with_execution_policy(ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::MIN,
        ))
    } else {
        resources
    };
    let limits = installation::WorthQueryApplicationLimits::new(
        resources,
        worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).unwrap(),
        ),
        worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget::development(),
    );
    installation::in_memory_program(
        program,
        schema,
        (calls,),
        installation::ApplicationBuildIdentity::parse("11111111-1111-4111-8111-111111111111")
            .unwrap(),
        limits,
        |_, graph, installed| {
            graph.bind_principal(
                &installed
                    .principal_binding(PrincipalBinding::reference())
                    .unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                world::identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Node::reference(),
                    WorthQueryApplicationEntityKey::new("root").unwrap(),
                )
                .field(Key::at("Key"), "root".to_owned())
                .field(ParentName::at("ParentName"), String::new())
                .field(Value::at("Value"), 1)
                .field(Width::at("Width"), 1)
                .field(Produced::at("Produced"), 0),
            )?;
            Ok(())
        },
    )
    .unwrap()
}
