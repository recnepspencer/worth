//! Actual abandonment must retain its own Native effect, not merely an error tag.
use super::*;
use host_observation::{comparison, fates::*, projection, receipts};
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::WorthQueryApplicationHomeLocation;

#[test]
fn abandonment_fate_correspondence_requires_original_effect_and_unmoved_product() {
    let path = tempfile::tempdir().unwrap();
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let homes: Vec<_> = [
        WorthQueryApplicationHomeLocation::Directory(path.path().to_path_buf()),
        WorthQueryApplicationHomeLocation::MemoryOnly,
    ]
    .into_iter()
    .map(|location| {
        host_observation::opening::open(
            location,
            opening::build(),
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
            None,
            Default::default(),
            &execution,
        )
    })
    .collect();
    let mut map = comparison::HostOracleComparison::default();
    let mut responses = Vec::new();
    for home in &homes {
        let before = projection::observe(home.runtime(), &execution);
        assert!(before.custody.is_empty());
        let outcome =
            host_interruption::publication::abandon_after_owner(home.runtime(), 6_301, 9_121);
        responses.push(receipts::observe(&outcome, &before.head));
        drop(outcome);
    }
    let mut states: Vec<_> = homes
        .iter()
        .map(|home| projection::observe(home.runtime(), &execution))
        .collect();
    FateCorrespondence::admit(
        map.identities_mut(),
        &responses[0],
        &responses[1],
        &states[0],
        &states[1],
    )
    .unwrap_or_else(|cause| {
        panic!("actual abandonment: {cause:?}; responses={responses:?}; states={states:?}")
    });
    comparison::compare(&mut map, &states[0], &states[1]).unwrap();

    let original = &states[0].custody[0]
        .original_effect
        .as_ref()
        .unwrap()
        .native;
    let reference = &states[1].custody[0]
        .original_effect
        .as_ref()
        .unwrap()
        .native;
    let mut wrong = serde_json::to_value(original).unwrap();
    wrong["validation"][0]["metadata"]["version"] =
        serde_json::to_value(states[0].entities[0].created_at_version).unwrap();
    let wrong = serde_json::from_value(wrong).unwrap();
    assert!(
        host_observation::native_semantic::compare(map.identities_mut(), &wrong, reference)
            .is_err(),
        "validation bases cannot acquire a second correspondence for a graph version"
    );
    let mut wrong = serde_json::to_value(original).unwrap();
    let previous = wrong["change"]["adjacency_delta_count"].as_u64().unwrap();
    wrong["change"]["adjacency_delta_count"] = serde_json::json!(previous + 1);
    let wrong = serde_json::from_value(wrong).unwrap();
    assert!(
        host_observation::native_semantic::compare(map.identities_mut(), &wrong, reference)
            .is_err(),
        "non-identity Native meaning remains exact"
    );

    let pending = responses[0].unresolved.as_mut().unwrap();
    let UnresolvedOrigin::RetainedPreparation { stage, .. } = &mut pending.origin else {
        panic!("actual preparation");
    };
    *stage = PreparationStage::PreparingNative;
    assert!(
        FateCorrespondence::admit(
            map.identities_mut(),
            &responses[0],
            &responses[1],
            &states[0],
            &states[1]
        )
        .is_err(),
        "a pre-effect preparation cannot borrow an after-effect fate"
    );
    let UnresolvedOrigin::RetainedPreparation { stage, .. } =
        &mut responses[0].unresolved.as_mut().unwrap().origin
    else {
        panic!("actual preparation");
    };
    *stage = PreparationStage::Publishing;
    let effect = states[0].custody[0].original_effect.take().unwrap();
    assert!(
        FateCorrespondence::admit(
            map.identities_mut(),
            &responses[0],
            &responses[1],
            &states[0],
            &states[1]
        )
        .is_err(),
        "a count cannot replace the original Native association"
    );
    states[0].custody[0].original_effect = Some(effect);
    states[0].custody[0]
        .original_effect
        .as_mut()
        .unwrap()
        .attempt += 1;
    assert!(
        FateCorrespondence::admit(
            map.identities_mut(),
            &responses[0],
            &responses[1],
            &states[0],
            &states[1]
        )
        .is_err(),
        "a different World attempt cannot acquire the already-compared Native effect"
    );
    states[0].custody[0]
        .original_effect
        .as_mut()
        .unwrap()
        .attempt -= 1;
    states[0].head.generation += 1;
    assert!(
        FateCorrespondence::admit(
            map.identities_mut(),
            &responses[0],
            &responses[1],
            &states[0],
            &states[1]
        )
        .is_err(),
        "a moved product is a different fate"
    );
}
