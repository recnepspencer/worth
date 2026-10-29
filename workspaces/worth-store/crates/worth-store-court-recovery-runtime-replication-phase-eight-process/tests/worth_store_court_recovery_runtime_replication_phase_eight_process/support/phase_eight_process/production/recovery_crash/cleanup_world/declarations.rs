pub(crate) use {crate::phase_eight_process::production::harness::ProcessWorld};

pub(crate) fn require_raw_candidate(world: &ProcessWorld) {
    world.require_cleanup_candidate().unwrap_or_else(|error| {
        panic!("cleanup world lacks a checkpoint-covered WAL artifact: {error}")
    });
}