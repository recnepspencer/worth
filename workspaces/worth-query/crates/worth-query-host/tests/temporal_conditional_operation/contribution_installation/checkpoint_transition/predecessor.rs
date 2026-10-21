//! Creates the actual predecessor checkpoint for transition tests.
use super::*;

pub(super) struct LegacyTemporalProgram;
impl ApplicationProgramDefinition<TemporalHostSchema> for LegacyTemporalProgram {
    type Contributions = <TemporalHostSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.host.temporal-installation.legacy");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        TemporalInstallationProgram::feature_specs()
    }
}
pub(super) fn configuration() -> (TemporalContributionConfiguration,) {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    (TemporalContributionConfiguration {
        installation_predicate,
        definition_predicate: Arc::new(definition_predicate),
        clock_source,
        clock_control,
        contacts,
        install_route: true,
    },)
}

pub(super) fn source() -> (
    application_installation::WorthQueryApplicationCheckpoint,
    Predecessor,
) {
    let program = ApplicationProgramAuthoring::<TemporalHostSchema, LegacyTemporalProgram>::begin()
        .validated_program()
        .unwrap();
    let predecessor = Predecessor::new(&program.revision().to_string()).unwrap();
    let application = application_installation::in_memory_program(
        program,
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint::checkpoint_limits(),
        |_phase, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "checkpoint-transition", 0, 1, true);
            Ok(())
        },
    )
    .unwrap();
    (
        application
            .with_host_advancement(|phase| {
                application
                    .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                    .unwrap()
            })
            .expect("installed host call opens"),
        predecessor,
    )
}
