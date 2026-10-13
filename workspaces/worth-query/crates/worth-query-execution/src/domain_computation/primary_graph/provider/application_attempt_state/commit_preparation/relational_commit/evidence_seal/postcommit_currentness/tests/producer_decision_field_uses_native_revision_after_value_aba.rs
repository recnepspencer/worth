//! producer decision field uses native revision after value aba.
use super::*;

#[test]
fn producer_decision_field_uses_native_revision_after_value_aba() {
    let world = installed_authorization_world(true);
    let (entity, locator) = world
        .application
        .with_host_advancement(|phase| account_note(&phase, &world, "account-1"))
        .expect("the fixture read owns its installed host call");
    let field = WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind: world
            .application
            .runtime
            .primary_graph()
            .unwrap()
            .layout
            .entity_kind(AccountIdentity::reference().entity())
            .unwrap(),
        locator: planned(&locator),
        value: StringApplicationValueBinding::encode(&"reviewed".to_owned()).unwrap(),
    };
    let facts = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            rebase_at_current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                vec![field.clone()],
            )
        })
        .expect("the fixture read owns its installed host call");
    let durable = crate::domain_computation::primary_graph::application_checkpoint::decode_producer_facts(
            &crate::domain_computation::primary_graph::application_checkpoint::encode_producer_facts(&facts)
                .expect("the rebased non-query producer decision field is checkpoint-comparable"),
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the complete producer fact set round-trips");
    assert_eq!(durable.as_ref(), facts.as_ref());
    assert!(matches!(
        facts[0],
        WorthQueryApplicationObservedFact::SourceFieldRevision {
            native_revision: Some(_),
            ..
        }
    ));
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                &facts[0]
            ))
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            assert!(matches!(
                output_selection(&phase, &phase.borrowed_request_for_test(), &world, &durable),
                OutputDependencySelection::Reuse
            ))
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                currentness(&phase, &phase.borrowed_request_for_test(), &world, &field),
                Err(WorthQuerySourceCurrentnessFailure::Unavailable),
                "a value-only fact cannot authorize output reuse"
            )
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            set_note(
                &host_custody.borrowed_request_for_test(),
                &world,
                entity,
                locator.clone(),
                "temporary",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            set_note(
                &host_custody.borrowed_request_for_test(),
                &world,
                entity,
                locator,
                "reviewed",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(!current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                &facts[0]
            ))
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            assert!(matches!(
                output_selection(&phase, &phase.borrowed_request_for_test(), &world, &durable),
                OutputDependencySelection::FreshRequired
            ))
        })
        .expect("the fixture read owns its installed host call");
}
