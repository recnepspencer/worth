//! A genuine declared program installs authored conditionals without a temporal runtime.
use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
use worth_query_host::facade::{WorthQueryApplicationHome,WorthQueryApplicationHomeLocation,WorthQueryApplicationHomeRequest,
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::{ApplicationBuildIdentity,WorthQueryApplicationProgramRoster}};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::*;
pub(super) type Home = WorthQueryApplicationHome<TemporalHostSchema, TemporalInstallationProgram>;
pub(super) fn open(
    location: WorthQueryApplicationHomeLocation,
    build: ApplicationBuildIdentity,
    initialized: Arc<AtomicUsize>,
    contacts: ContactCounters,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Home {
    let declaration = TemporalHostSchema::declaration().unwrap();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let configuration = TemporalContributionConfiguration {
        installation_predicate,
        definition_predicate: Arc::new(definition_predicate),
        clock_source,
        clock_control,
        contacts,
        install_route: true,
        definition_only: true,
    };
    let request = WorthQueryApplicationHomeRequest::new(
        location,
        ApplicationProgramAuthoring::<TemporalHostSchema, TemporalInstallationProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (configuration,),
        build,
        publication_limit::limits(),
        move |_, graph, installed| {
            initialized.fetch_add(1, Ordering::SeqCst);
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "blocked", 0, 1, true);
            Ok(())
        },
    );
    with_bootstrap_advancement_from_request(*execution, |phase| {
        Box::new(request.open_in_advancement(&phase))
    })
    .expect("actual bootstrap request admits the opening")
    .unwrap_or_else(|refused| panic!("authored Signal Host opening: {refused:?}"))
}
