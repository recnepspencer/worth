//! Definition admission installs no managed clock, temporal intent or invoker.
use super::*;
pub fn installed_definition_keeps_temporal_execution_closed() {
    let mut configuration = publication_limit::configuration();
    configuration.definition_only = true;
    let application = application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (configuration,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        WorthQueryApplicationLimits::new(
            product_world_resources(1_024),
            runtime::WorthQueryApplicationQueryResourceProfile::new(
                std::num::NonZeroUsize::new(128).unwrap(),
            ),
            primary_graph::SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "blocked", 0, 1, true);
            Ok(())
        },
    )
    .unwrap();
    let installed = application.conditional::<TemporalConditional>().unwrap();
    let installed = installed.as_ref().as_ref().unwrap();
    assert_eq!(installed.clock_control.observations(), 0);
    assert_eq!(installed.contacts.snapshot(), (0, 0, 0, 0));
    application.with_host_advancement(|phase| {
        let selected = application.on_branch(application.current_world()).select(&phase).unwrap();
        let request = phase.checked_execution_request_for(application.product_runtime()).unwrap();
        let denial = match selected.conditional_clock(&request, &installed.clock) {
            Err(denial) => denial,
            Ok(_) => panic!("definition-only membership admitted a managed clock"),
        };
        assert!(matches!(denial.kind(),
            primary_graph::WorthQueryConditionalClockObservationDenialKind::NotReadmitted {
                family: worth_foundational::facade::DurableStateFamily::BridgeManagedTemporalIntents, phase: 7,
            }));
    }).unwrap();
    assert_eq!(installed.clock_control.observations(), 0);
    assert_eq!(installed.contacts.snapshot(), (0, 0, 0, 0));
}
