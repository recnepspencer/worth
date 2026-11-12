//! Equal output from a new installed implementation must physically publish.
use super::super::failure;
use super::*;
use worth_relational::facade::inspection::observe_maintenance_work;
fn settle(
    host: &WorthQueryProgramApplicationRuntime<Schema, program::Lazy>,
    request: &family::Request<'_, '_, '_>,
    maximum_contacts: u64,
) {
    let started = host.producer_contacts_on_this_thread_for_test();
    let mut pending = request
        .advanced()
        .start_program_outputs::<program::Lazy, program::LazyTree>(
            host,
            Demand("a".into()),
            WorthQueryOutputDemandControls::default(),
        )
        .unwrap();
    for _ in 0..128 {
        let progress = pending.advance(host, request);
        let actual = host.producer_contacts_on_this_thread_for_test() - started;
        assert!(actual <= maximum_contacts,
            "an implementation upgrade must retain its new identity within one execution per route: actual={actual} maximum={maximum_contacts}");
        if matches!(
            progress.unwrap(),
            WorthQueryApplicationProgramOutputProgress::Settled(_)
        ) {
            return;
        }
    }
    panic!("upgraded producer tree must settle within its request bound");
}
fn run(worker: bool) {
    let mut saved = None;
    courts::with_legacy_rows(worker, |host, _, expected| {
        saved = Some((failure::checkpoint(host), expected));
    });
    let (checkpoint, expected) = saved.unwrap();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::restore_program_checkpoint::<program::Lazy>(checkpoint, worker, true);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_, published) = observe_maintenance_work(16, || settle(&host, &request, 3));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        3,
        "new installed implementation executes the root and both child routes"
    );
    assert_eq!(
        published.published_rows.len() + published.omitted_publications,
        3,
        "implementation changes refuse adoption even when every model row is equal"
    );
    assert_eq!(
        request
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0],
        expected
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_, ready) = observe_maintenance_work(8, || settle(&host, &request, 0));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "the actual upgrade publication is Current on the next demand"
    );
    assert_eq!(ready.published_rows.len() + ready.omitted_publications, 0);
}
#[test]
fn equal_outputs_publish_after_an_implementation_upgrade() {
    run(false);
}
#[test]
fn worker_only_equal_outputs_publish_after_an_implementation_upgrade() {
    run(true);
}
