//! Genuine co-anchors survive retirement, fresh restoration and another retirement.
mod journey;
use super::*;
use worth_relational::facade::history::CommitId;

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
    // Decode the real text directly: serde_json::Value cannot carry Native u128 IDs.
    #[derive(serde::Deserialize)]
    struct Retired {
        anchor: EntityId,
        co_anchors: Vec<(EntityId, CommitId)>,
    }
    #[derive(serde::Deserialize)]
    struct Restored {
        anchor: EntityId,
        co_anchors: Vec<(EntityId, EntityId)>,
    }
    #[derive(serde::Deserialize)]
    struct RetirementEnvelope {
        authored_cascade_retirement: Retired,
    }
    #[derive(serde::Deserialize)]
    struct RestorationEnvelope {
        authored_cascade_restoration: Restored,
    }
    let retired: RetirementEnvelope = serde_json::from_str(
        &serde_json::to_string(world.retired.publication().envelope.as_ref()).unwrap(),
    )
    .unwrap();
    let restored: RestorationEnvelope = serde_json::from_str(
        &serde_json::to_string(world.restored.publication().envelope.as_ref()).unwrap(),
    )
    .unwrap();
    let retired = retired.authored_cascade_retirement;
    let restored = restored.authored_cascade_restoration;
    assert_eq!(restored.anchor, retired.anchor);
    assert_eq!(
        restored
            .co_anchors
            .iter()
            .map(|pair| pair.1)
            .collect::<std::collections::BTreeSet<_>>(),
        retired
            .co_anchors
            .iter()
            .map(|pair| pair.0)
            .collect::<std::collections::BTreeSet<_>>(),
    );
    let co_anchors = retired
        .co_anchors
        .iter()
        .enumerate()
        .map(|(i, pair)| (pair.0, targets(&[&format!("inverse-co-{i}")]).remove(0)))
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
