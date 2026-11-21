//! A checkpoint retains only actual external component obligations.
//! Restored C2 roster census of
//! 0615dd733c:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs
//! (`recorded_component_checkpoint_omits_released_components_and_plain_observations`).
//! Fixture adaptations: the same-SHA `original` fixture is the testkit's
//! `original` over `TestDurableMedium`; the private directory decode is the
//! owner's `inspect_component_roster`. The other C2 portions of that file are
//! restored in place in `recorded_original_checkpoint_journeys.rs`.
//! `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`
//! needs a boundary ruling (see the C2 capture progress note): its fixture
//! holds the owner's private external cut and lane gate.
use super::*;
use std::sync::Arc;
use worth_relational::facade::durability::RelationalDurableColdSource;

#[test]
fn recorded_component_checkpoint_omits_released_components_and_plain_observations() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, false);
    let source: Arc<dyn RelationalDurableColdSource> = original.medium.clone();
    let checkpoint = capture(execution, &original.source);
    original.medium.file_checkpoint(checkpoint.clone());
    let roster = checkpoint
        .inspect_component_roster(execution, Some(source.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(roster.retained_roots, 1);
    assert_eq!(roster.retained_empty_components, 0);
    assert!(!roster
        .recorded_components
        .iter()
        .any(|component| component == original.before.descriptor()
            || component == original.after.descriptor()));

    // An ordinary observation has no external durable component obligation.
    let (_, observed) = original
        .source
        .observe_branch(&original.source.main_branch_identity())
        .unwrap();
    create_entity(
        execution,
        &original.source,
        "observation-alone-is-not-a-component",
    );
    let next = capture(execution, &original.source);
    original.medium.file_checkpoint(next.clone());
    let roster = next
        .inspect_component_roster(execution, Some(source))
        .unwrap()
        .unwrap();
    assert_eq!(roster.retained_roots, 1);
    drop(observed);
}
