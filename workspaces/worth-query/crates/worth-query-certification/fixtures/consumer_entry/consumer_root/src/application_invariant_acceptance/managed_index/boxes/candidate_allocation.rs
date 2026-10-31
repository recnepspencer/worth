//! Isolated physical allocation across actual candidate collection before checked root pruning.
use super::super::super::authentication;
use super::super::{host, lifecycle, prefix};
use super::*;
use std::sync::{atomic::Ordering, Arc, Mutex};
use worth_query_host::facade::application_entry::*;
#[test]
fn pruned_completed_source_allocates_no_dense_candidate_backing() {
    lifecycle::isolated_allocation_case(
        journey,
        concat!(
            module_path!(),
            "::pruned_completed_source_allocates_no_dense_candidate_backing"
        ),
    );
}
fn journey() {
    let calls = Arc::new(Calls::default());
    let world = host::install_boxes(Arc::new(prefix::Calls::default()), calls.clone());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let probe = Coordinates::new(100, 100, 101, 101);
    let input = || geometry::input("boxes", probe);
    let source = request
        .query(query::ReadSource {
            directory: "boxes".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 6, "actual nonempty Completed Source");
    let index = request.query(input()).open_index::<BoxIndex>().unwrap();
    request.query(input()).rebuild_index(&index).unwrap();
    let samples = Arc::new(Mutex::new([None; 2]));
    let observed = samples.clone();
    *calls.before_candidates.lock().unwrap() = Some(Box::new(move || {
        observed.lock().unwrap()[0] = Some(lifecycle::ALLOCATOR.stats());
    }));
    let observed = samples.clone();
    *calls.first_summary.lock().unwrap() = Some(Box::new(move || {
        observed.lock().unwrap()[1] = Some(lifecycle::ALLOCATOR.stats());
    }));
    let comparisons = calls.summary_comparisons.load(Ordering::SeqCst);
    let matches = calls.matched_sources.load(Ordering::SeqCst);
    let result = request.query(input()).execute();
    assert!(result.unwrap().rows().is_empty());
    assert_eq!(
        calls.summary_comparisons.load(Ordering::SeqCst),
        comparisons + 1
    );
    assert_eq!(calls.matched_sources.load(Ordering::SeqCst), matches);
    let samples = samples.lock().unwrap();
    let before = samples[0].expect("actual probe callback");
    let after = samples[1].expect("actual first-summary callback");
    let interval = after - before;
    println!("physical candidate-allocation interval: {interval:?}");
    assert_eq!(
        interval.allocations, 0,
        "pruned empty collection allocates no backing"
    );
    assert_eq!(
        interval.bytes_allocated, 0,
        "no Source-sized candidate bitmap"
    );
    drop(samples);
    index.close();
}
