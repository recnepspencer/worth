//! Creates the actual predecessor Directory Home for D16 program transition tests.
pub use super::*;
use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

pub struct LegacyTemporalProgram;
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
pub fn configuration() -> (TemporalContributionConfiguration,) {
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
        definition_only: false,
        multi_intent_targets: false,
    },)
}

pub fn transition_resources(
) -> worth_relational::facade::indexes::DerivedIndexMaintenanceBudget {
    worth_relational::facade::indexes::DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(500_000),
        maximum_cold_record_slots: Some(8_192),
        maximum_derived_rows: Some(8_192),
    }
}

pub fn predecessor_request(
    location: std::path::PathBuf,
) -> WorthQueryApplicationHomeRequest<'static, TemporalHostSchema, LegacyTemporalProgram> {
    let program = ApplicationProgramAuthoring::<TemporalHostSchema, LegacyTemporalProgram>::begin()
        .validated_program()
        .unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        program,
        application_installation::WorthQueryApplicationProgramRoster::new(),
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
}

pub fn successor_request(
    location: std::path::PathBuf,
) -> WorthQueryApplicationHomeRequest<'static, TemporalHostSchema, TemporalInstallationProgram> {
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        validated_program(),
        application_installation::WorthQueryApplicationProgramRoster::new(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint::checkpoint_limits(),
        |_, _, _| panic!("existing Home cannot initialize during successor open"),
    )
}

pub fn open_predecessor_directory(
    location: std::path::PathBuf,
) -> WorthQueryApplicationHome<TemporalHostSchema, LegacyTemporalProgram> {
    let lease = crate::facade::host::checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request(
        execution,
        |phase| predecessor_request(location).open_in_advancement(&phase),
    )
    .expect("bootstrap opens")
    .expect("predecessor Directory Home opens")
}
