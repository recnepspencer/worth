//! publishes delivers and executes.
use super::*;

pub(in super::super) fn publishes_delivers_and_executes(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let application = super::application_fixture::installed_application();
    assert!(application
        .installed_program()
        .derived_artifact::<
            ApplicationRootComposition,
            TemporalInstallationFeature,
            TemporalArtifact,
        >()
        .is_some());
    let installed = application
        .conditional::<TemporalConditional>()
        .expect("the installed conditional handle must remain typed and reachable");
    let installed = installed
        .as_ref()
        .as_ref()
        .expect("the admitted conditional must retain its installed route");
    let branch = application.current_world();
    let selected = application
        .with_host_advancement(|phase| application.on_branch(branch).select(&phase).unwrap())
        .expect("installed host call opens");
    let definition = selected
        .publish_conditional_definition(
            &installed.clock,
            Arc::clone(&installed.definition_predicate),
            &runtime::RuntimeWorldCancellationSource::new().token(),
        )
        .expect("the contribution-installed route must publish a real definition");
    assert!(matches!(
        definition,
        primary_graph::WorthQueryConditionalDefinitionPublicationOutcome::Performed(_)
    ));
    drop(definition);

    let mut publication = change_input(
        request_execution,
        &application,
        &installed.invariant,
        branch,
        "changed",
    )
    .require_committed()
    .expect("the source mutation must publish");
    let change = publication
        .take_performed_relational_product_change()
        .expect("the source mutation must retain World's performed patch");
    let selected = application
        .with_host_advancement(|phase| application.on_branch(branch).select(&phase).unwrap())
        .expect("installed host call opens");
    let delivery = application
        .with_application_advancement(&request_scope(), |phase| {
            selected.deliver_relational_change_to_conditional(&phase, &installed.clock, 0, change)
        })
        .expect("the host request must be admitted")
        .expect("Bridge must admit the actual performed World patch");
    assert!(matches!(
        delivery,
        runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(_)
    ));
    drop(delivery);

    installed.clock_control.push(3, 11);
    let observation = selected
        .conditional_clock(request_execution, &installed.clock)
        .unwrap()
        .observe(request_execution);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(observation) =
        observation
    else {
        panic!("the contribution-installed conditional must execute")
    };
    assert_eq!(observation.committed_operation_count(), 1);
    assert_eq!(observation.authoritative_commit_count(), 1);
    assert_eq!(installed.contacts.snapshot(), (1, 1, 1, 1));
}
