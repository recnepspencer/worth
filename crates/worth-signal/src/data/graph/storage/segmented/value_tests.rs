use super::SegmentedStore;
use crate::data::graph::storage::handles::DependencySetId;

#[test]
fn equality_is_transparent_to_fork_and_operational_clone_posture() {
    let mut source = SegmentedStore::<u64, DependencySetId>::default();
    let inherited = source.insert_from_slice(&[1, 2]);
    let mut destination = source.fork_persistent();
    assert!(source.shares_storage_with(&destination));
    assert_eq!(source, destination);

    let inherited_clone = destination.clone();
    assert!(destination.shares_storage_with(&inherited_clone));
    assert_eq!(destination, inherited_clone);
    assert_eq!(inherited_clone.get(inherited), &[1, 2]);
    let inherited_reconstruction = destination.operational_clone();
    assert!(!destination.shares_storage_with(&inherited_reconstruction));
    assert_eq!(destination, inherited_reconstruction);

    let appended = destination.insert_from_slice(&[3]);
    assert_ne!(source, destination);
    let changed_clone = destination.clone();
    assert!(destination.shares_storage_with(&changed_clone));
    assert_eq!(destination, changed_clone);
    assert_eq!(changed_clone.get(inherited), &[1, 2]);
    assert_eq!(changed_clone.get(appended), &[3]);
    assert_eq!(source.live_segment_count(), 1);
    assert_eq!(source.get(inherited), &[1, 2]);
}

#[test]
fn equality_preserves_complete_deserialized_flat_layout() {
    let mut left: SegmentedStore<u64, DependencySetId> =
        serde_json::from_str(r#"{"items":[11,99],"segments":[{"start":0,"len":1}]}"#)
            .expect("left layout deserializes");
    let right: SegmentedStore<u64, DependencySetId> =
        serde_json::from_str(r#"{"items":[11,88],"segments":[{"start":0,"len":1}]}"#)
            .expect("right layout deserializes");
    assert_ne!(left, right);

    let fork = left.fork_persistent();
    assert_ne!(fork, right);
    assert_eq!(fork, fork.clone());
}
