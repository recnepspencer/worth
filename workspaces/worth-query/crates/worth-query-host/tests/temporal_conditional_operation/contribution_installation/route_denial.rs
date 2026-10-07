//! A missing contribution-installed route refuses publication.
use super::*;

pub(crate) fn zero_route_installation_is_denied() {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let result = application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (TemporalContributionConfiguration {
            installation_predicate,
            definition_predicate: Arc::new(definition_predicate),
            clock_source,
            clock_control,
            contacts,
            install_route: false,
        },),
        WorthQueryApplicationLimits::new(
            product_world_resources(1_024),
            runtime::WorthQueryApplicationCandidateResourceProfile::bounded(5_120, 2_048, 5_120)
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
        |_, _| Ok(()),
    );
    match result {
        Err(application_installation::WorthQueryApplicationOpenRefusal { denial: application_installation::WorthQueryApplicationOpenDenial::ConditionalPublication(denial), .. }) => assert_eq!(
            denial.kind(),
            primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::IncompleteBindingInventory,
        ),
        Err(other) => panic!("expected incomplete conditional route denial, got {other:?}"),
        Ok(_) => panic!("a zero-route conditional installation published an application"),
    }
}
