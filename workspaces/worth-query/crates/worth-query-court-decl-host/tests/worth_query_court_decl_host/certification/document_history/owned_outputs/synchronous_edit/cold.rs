//! Ordinary coupled publications reopen through the existing Native/currentness owners.
use super::*;

fn reopen(worker: bool) {
    let host = world::open_directory_host(worker);
    courts::edit_two_families(&host);
    let original = (
        world::model(&host, host.current_world()),
        world::outputs(&host, host.current_world()),
    );
    let directory = host.checkpoint_and_close_for_reopen();
    let host = world::reopen_same_build(directory, worker);
    assert_eq!(
        (
            world::model(&host, host.current_world()),
            world::outputs(&host, host.current_world())
        ),
        original
    );
    let before = courts::commit_ordinal(&host);
    let native_before = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "cold current outputs do not reenter either writer"
    );
    assert_eq!(
        courts::commit_ordinal(&host),
        before,
        "cold readiness is also a read"
    );
    assert_eq!(courts::native_head(&host), native_before);
    assert_eq!(world::outputs(&host, host.current_world()), original.1);
}

#[test]
fn ordinary_coupled_outputs_reopen_with_equal_current_facts() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    reopen(false);
}
#[test]
fn worker_only_ordinary_coupled_outputs_reopen_with_equal_current_facts() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    reopen(true);
}
