use super::{
    lifecycle::checkpoint_stage_label, markers, mutation_material::checkpoint_idempotency_material,
    Invocation,
};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, ServingPhysicalRuntime,
};

pub(super) fn complete(serving: &ServingPhysicalRuntime, seed: u64) -> Result<(), String> {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(checkpoint_idempotency_material(
            seed ^ 0xC8_00_00_01,
        )),
        PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .expect("C8 first checkpoint deadline is nonzero"),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        return Err("ordinary C8 first checkpoint admission was denied".to_owned());
    };
    if !matches!(handle.wait(), PhysicalCheckpointOutcome::Completed(_)) {
        return Err("ordinary C8 first checkpoint did not complete".to_owned());
    }
    Ok(())
}

pub(super) fn hold_at_stage(
    serving: &ServingPhysicalRuntime,
    invocation: &Invocation,
) -> Result<(), String> {
    let gate = serving.pause_physical_checkpoint_at(invocation.stage.stage.step());
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(checkpoint_idempotency_material(
            invocation.stage.perturbation_seed,
        )),
        PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .expect("C8 checkpoint deadline is nonzero"),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        return Err("ordinary C8 checkpoint admission was denied".to_owned());
    };
    if !gate.await_arrival() {
        return Err(format!(
            "ordinary C8 checkpoint did not reach its production seam: {:?}",
            handle.poll()
        ));
    }
    if invocation.stage.completes_after_arrival() {
        gate.release();
        if !matches!(handle.wait(), PhysicalCheckpointOutcome::Completed(_)) {
            return Err(
                "completed C8 checkpoint did not finish after namespace synchronization".to_owned(),
            );
        }
    } else {
        let _checkpoint_handle = handle;
    }
    let stage = checkpoint_stage_label(invocation.stage.stage);
    markers::write_reached(
        &invocation.reached_marker,
        stage.as_bytes(),
        "write C8 checkpoint reached marker",
    )?;
    markers::park_forever();
}
