//! A present optional source denies an equal value after ABA.
use super::*;

#[test]
fn present_optional_source_denies_equal_value_after_aba() {
    let world = installed_authorization_world(true);
    let (before, fact, account) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            execute_with_note_fact(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                "account-1",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(before.note(), Some("reviewed"));
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let note = AccountNote::reference();
    let locator = (*graph.layout_for_test())
        .field_locator_for_test(note.entity(), note.aspect(), note.field())
        .unwrap()
        .clone();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            set_string_field(
                &host_custody.borrowed_request_for_test(),
                &world,
                account,
                locator.clone(),
                "temporary",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            set_string_field(
                &host_custody.borrowed_request_for_test(),
                &world,
                account,
                locator,
                "reviewed",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                execute(&phase, &world, "account-1").note(),
                Some("reviewed")
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(
                !current(
                    &phase,
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    &fact
                ),
                "returning to the same value cannot restore the old native revision"
            )
        })
        .expect("the fixture read owns its installed host call");
}
