use worth_runtime_world::facade::{ProductBranchObservation, ProductBranchObservationMismatchAxis};

fn assert_observation_contract<T>()
where
    T: Clone + Send + Sync + std::fmt::Debug,
{
}

#[test]
fn managed_observation_is_cloneable_and_sendable() {
    assert_observation_contract::<ProductBranchObservation>();
}

#[test]
fn reference_comparison_vocabulary_keeps_each_dynamic_axis_distinct() {
    let axes = [
        ProductBranchObservationMismatchAxis::OwnerIdentity,
        ProductBranchObservationMismatchAxis::BranchIdentity,
        ProductBranchObservationMismatchAxis::LifecycleIncarnation,
        ProductBranchObservationMismatchAxis::ReferenceGeneration,
        ProductBranchObservationMismatchAxis::SelectedCompositeCommit,
        ProductBranchObservationMismatchAxis::CompositeBasis,
    ];

    for (index, axis) in axes.iter().enumerate() {
        assert!(axes[..index].iter().all(|prior| prior != axis));
    }
}
