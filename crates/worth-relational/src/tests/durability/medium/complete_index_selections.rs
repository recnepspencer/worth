//! A Complete capture of a never-demanded image keeps the selections that image carried.
//!
//! Kept in-source (listed as needing a door): it observes that capture leaves the
//! deferred image un-admitted, which no public owner exposes. The carried-selection
//! journey itself is proven publicly by `worth-court-relational-certification`
//! `incremental_indexes/complete_carry.rs` and the Directory Home reopen courts.
use super::*;

#[test]
fn reclose_before_any_image_demand_keeps_the_selections_the_image_carried() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    // An actual reopened owner whose index image is installed and never demanded.
    let fixture = super::cold_index_fixture::reopened_carried(execution);
    let pending = fixture.runtime.indexes.carried_checkpoint_image();
    assert_eq!(pending, Some(fixture.member));
    let original = fixture
        .runtime
        .indexes
        .checkpoint_inputs_for_request(execution)
        .unwrap()
        .resolve_for_request(execution)
        .unwrap();
    assert_eq!(
        fixture.runtime.indexes.carried_checkpoint_image(),
        pending,
        "capture reads the original image without admitting it"
    );
    let carried = fixture.carried.as_ref().unwrap();
    let head = original
        .bases()
        .iter()
        .find(|basis| basis.version == fixture.generation.applicability.version_id)
        .expect("the image's current-root selection is carried into the next capture");
    assert_eq!(head.commit, fixture.generation.source_commit_id);
    let mut expected = vec![
        (fixture.index, fixture.generation.generation_id),
        (carried.index_id, carried.generation_id),
    ];
    expected.sort_unstable();
    assert_eq!(head.bindings, expected);
    for generation in [&fixture.generation, carried] {
        assert!(original
            .generations()
            .iter()
            .any(|emitted| emitted == generation));
    }
}
