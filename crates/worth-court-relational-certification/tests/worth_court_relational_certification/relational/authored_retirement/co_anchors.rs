//! Genuine co-anchors survive retirement, fresh restoration and another retirement.
mod journey;
use super::*;

#[test]
fn co_anchored_history_preserves_fresh_restoration_and_nonconsuming_inverse_admission() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let world = journey::run(execution);
    let runtime = &world.runtime;
    let basis = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let retired = serde_json::to_value(world.retired.publication().envelope.as_ref()).unwrap();
    let restored = serde_json::to_value(world.restored.publication().envelope.as_ref()).unwrap();
    let retired = &retired["authored_cascade_retirement"];
    let restored = &restored["authored_cascade_restoration"];
    assert_eq!(restored["anchor"], retired["anchor"]);
    assert_eq!(
        restored["co_anchors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| serde_json::from_value::<EntityId>(pair[1].clone()).unwrap())
            .collect::<std::collections::BTreeSet<_>>(),
        retired["co_anchors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| serde_json::from_value::<EntityId>(pair[0].clone()).unwrap())
            .collect::<std::collections::BTreeSet<_>>(),
    );
    let co_anchors = retired["co_anchors"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, pair)| {
            (
                serde_json::from_value::<EntityId>(pair[0].clone()).unwrap(),
                targets(&[&format!("inverse-co-{i}")]).remove(0),
            )
        })
        .collect::<Vec<_>>();
    let admit = || {
        runtime.prepare_authored_source_restoration_with_co_anchors(
            execution,
            &world.installed,
            &basis,
            world.retired.commit.commit_id,
            (world.source, targets(&["inverse-anchor"]).remove(0)),
            &co_anchors,
        )
    };
    assert!(admit().expect("lawful inverse admission").is_some());
    assert!(admit()
        .expect("nonconsuming admission remains available")
        .is_some());
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
