//! producer decision absence uses native presence revision.
use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::tests::*;

#[test]
fn producer_decision_absence_uses_native_presence_revision() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (entity, locator) = world
        .application
        .with_host_advancement(|phase| account_note(&phase, &world, "account-2"))
        .expect("the fixture read owns its installed host call");
    let fact = WorthQueryApplicationObservedFact::AbsentField {
        entity_id: entity,
        kind: (*(*world.application.runtime_for_test())
            .primary_graph()
            .unwrap()
            .layout_for_test())
        .entity_kind_for_test(AccountIdentity::reference().entity())
        .unwrap(),
        locator: planned(&locator),
    };
    let facts = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
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
            let host_custody = phase.test_host_request_for_test();
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
                output_selection(&phase, &phase.borrowed_request_for_test(), &world, &facts),
                OutputDependencySelection::Reuse
            ))
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
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
            let host_custody = phase.test_host_request_for_test();
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
                output_selection(&phase, &phase.borrowed_request_for_test(), &world, &facts),
                OutputDependencySelection::FreshRequired
            ))
        })
        .expect("the fixture read owns its installed host call");
}
