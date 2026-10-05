use crate::facade::{
    BoundedSignalInputs, DeclaredSignalInput, NodeContract, NodeId, SignalRuntimePolicy,
};
use crate::tests::support::ASPECT_A;

pub(super) fn worker_matrix() -> [(&'static str, usize); 3] {
    [("one worker", 1), ("two workers", 2), ("four workers", 4)]
}

pub(super) fn bounded_contract(inputs: &[NodeId]) -> NodeContract {
    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new(
        inputs
            .iter()
            .map(|&node| DeclaredSignalInput::new(node, ASPECT_A)),
    ))
}

pub(super) fn aggressive_parallel_runtime_policy() -> SignalRuntimePolicy {
    SignalRuntimePolicy::operational()
        .with_observation_activation(worth_foundational::ObservationActivationProfile::Continuous)
        .with_parallel_admission(crate::runtime_policy::ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 1,
            balanced_min_parallel_tasks: 1,
            latency_bounded_min_parallel_tasks: 1,
            full_parallel_min_tasks: 1,
        })
}
