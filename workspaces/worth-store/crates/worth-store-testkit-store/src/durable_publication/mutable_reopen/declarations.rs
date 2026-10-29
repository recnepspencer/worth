pub use {sha2::Digest, sha2::Sha256};
pub use {worth_proof::TransitionOutcome};
pub use {worth_signal::facade::TemporalDuration};
pub use {worth_store::physical_runtime::CompletedPhysicalCheckpoint, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::ServingPhysicalRuntime};

pub fn checkpoint_for_mutable_reopen(
    serving: &ServingPhysicalRuntime,
    scenario: &'static str,
) -> CompletedPhysicalCheckpoint {
    let mut digest = sha2::Sha256::new();
    digest.update(b"worth-store.integration-mutable-reopen-checkpoint.v1");
    digest.update(scenario.as_bytes());
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(digest.finalize().into()),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(10_000)
                .expect("the integration checkpoint deadline is nonzero"),
        ),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Denied(cause) => panic!("mutable-reopen checkpoint denied: {cause:?}"),
        worth_proof::TransitionOutcome::Deferred(cause) => {
            panic!("mutable-reopen checkpoint deferred: {cause:?}")
        }
        worth_proof::TransitionOutcome::Stale(cause) => panic!("mutable-reopen checkpoint stale: {cause:?}"),
        worth_proof::TransitionOutcome::Failed(cause) => {
            panic!("mutable-reopen checkpoint failed to start: {cause:?}")
        }
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("mutable-reopen checkpoint did not complete: {other:?}"),
    }
}