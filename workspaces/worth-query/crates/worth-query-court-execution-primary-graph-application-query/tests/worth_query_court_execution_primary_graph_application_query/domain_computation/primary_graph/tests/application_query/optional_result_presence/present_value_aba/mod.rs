//! A present optional source denies an equal value after ABA.
use super::*;

#[test]
fn present_optional_source_denies_equal_value_after_aba() {
    let world = installed_authorization_world(true, WorthQueryAdmissionClock::system());
    let (before, fact, account) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            execute_with_note_fact(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                "account-1",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(before.note(), Some("reviewed"));
    let graph = world.application.runtime.primary_graph().unwrap();
    let note = AccountNote::reference();
    let locator = graph
        .layout
        .field_locator(note.entity(), note.aspect(), note.field())
        .unwrap()
        .clone();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
