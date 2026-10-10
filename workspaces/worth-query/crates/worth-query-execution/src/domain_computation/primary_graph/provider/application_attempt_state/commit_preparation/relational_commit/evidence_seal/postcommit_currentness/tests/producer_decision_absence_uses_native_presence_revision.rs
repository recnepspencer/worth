//! producer decision absence uses native presence revision.
use super::*;

#[test]
fn producer_decision_absence_uses_native_presence_revision() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let (entity, locator) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            account_note(&phase, &world, "account-2")
        })
        .expect("the fixture read owns its installed host call");
    let fact = WorthQueryApplicationObservedFact::AbsentField {
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
    };
    let facts = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            rebase_at_current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                vec![fact],
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        facts[0],
        WorthQueryApplicationObservedFact::SourceFieldRevision {
            native_revision: Some(revision),
            ..
        } if revision.presence() == worth_relational::facade::runtime::RelationalFieldPresence::Absent
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
            let host_custody = phase.test_host_request();
            assert!(matches!(
                output_selection(&phase, execution, &world, &facts),
                OutputDependencySelection::Reuse
            ))
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
                "now-present",
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
            let host_custody = phase.test_host_request();
            assert!(matches!(
                output_selection(&phase, execution, &world, &facts),
                OutputDependencySelection::FreshRequired
            ))
        })
        .expect("the fixture read owns its installed host call");
}
