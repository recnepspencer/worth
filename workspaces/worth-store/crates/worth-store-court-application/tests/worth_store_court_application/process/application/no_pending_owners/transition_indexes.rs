//! Private definitions join the actual positioned Home Target effect.
use worth_execution::ExecutionRequest;
mod promotion;
mod publication;
mod target;

enum Journey {
    Publish,
    Abandon,
    ChangedCatalog,
    IncompleteGenerations,
}
#[test]
fn home_target_indexes_publish_the_prepared_catalog_and_exact_generations() {
    run(Journey::Publish);
}
#[test]
fn abandoned_home_target_preserves_the_installed_index_catalog_and_history() {
    run(Journey::Abandon);
}
#[test]
fn home_target_refuses_a_changed_index_catalog_before_native_effect() {
    run(Journey::ChangedCatalog);
}

#[test]
fn home_target_refuses_an_incomplete_generation_batch_before_native_effect() {
    run(Journey::IncompleteGenerations);
}

fn run(journey: Journey) {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    drop(publication::run(journey, &execution));
}

#[test]
fn a_retained_target_index_definition_holds_its_original_backing_after_home_abandonment() {
    let lease =
        worth_query_testkit_host::facade::host::checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    let retained = publication::run(Journey::Publish, &execution).unwrap();
    assert!(
        lease.charged_memory_bytes() > 0,
        "the escaped installed definition retains its original charge"
    );
    let last = retained.clone();
    drop(retained);
    assert!(
        lease.charged_memory_bytes() > 0,
        "the final definition clone owns that same backing"
    );
    drop(last);
    assert_eq!(
        lease.charged_memory_bytes(),
        0,
        "abandoned Home and final definition release all original request backing"
    );
}
