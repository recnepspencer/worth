//! The installed temporal program shared by public commit proofs.
use super::*;
pub(super) fn installed_application(
) -> application_installation::WorthQueryProgramApplicationRuntime<
    TemporalHostSchema,
    TemporalInstallationProgram,
> {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let configuration = TemporalContributionConfiguration {
        installation_predicate,
        definition_predicate: Arc::new(definition_predicate),
        clock_source,
        clock_control,
        contacts: contacts.clone(),
        install_route: true,
    };
    application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (configuration,),
        WorthQueryInMemoryApplicationLimits::new(
            product_world_resources(1_024),
            runtime::WorthQueryApplicationCandidateResourceProfile::physical_resources(
                5_120, 2_048,
            )
            .unwrap(),
            runtime::WorthQueryApplicationQueryResourceProfile::bounded(
                5_120,
                2_048,
                usize::MAX,
                128,
            )
            .unwrap(),
            primary_graph::SignalConditionalEvaluationBudget::development(),
        ),
        |_phase, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "blocked", 0, 1, true);
            Ok(())
        },
    )
    .expect("the contribution-composed temporal application must install")
}
