#[allow(unused_imports)]
use crate::reference::ordinary_context::*;
use {
    worth_runtime_world::facade::ProductBranchObservation,
    worth_runtime_world::facade::ProductBranchObservationMismatchAxis,
};

#[test]
fn managed_observation_is_cloneable_and_sendable() {
    assert_observation_contract::<ProductBranchObservation>();
}

#[test]
fn reference_comparison_vocabulary_keeps_each_dynamic_axis_distinct() {
    let axes = [
        worth_runtime_world::facade::ProductBranchObservationMismatchAxis::OwnerIdentity,
        worth_runtime_world::facade::ProductBranchObservationMismatchAxis::BranchIdentity,
        worth_runtime_world::facade::ProductBranchObservationMismatchAxis::LifecycleIncarnation,
        worth_runtime_world::facade::ProductBranchObservationMismatchAxis::ReferenceGeneration,
        worth_runtime_world::facade::ProductBranchObservationMismatchAxis::SelectedCompositeCommit,
        worth_runtime_world::facade::ProductBranchObservationMismatchAxis::CompositeBasis,
    ];

    for (index, axis) in axes.iter().enumerate() {
        assert!(axes[..index].iter().all(|prior| prior != axis));
    }
}
