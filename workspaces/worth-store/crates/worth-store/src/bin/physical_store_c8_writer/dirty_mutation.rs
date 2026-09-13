use super::{
    configuration, markers, mutation_material::dirty_material,
    mutation_submission::start_dirty_checkpoint, Invocation,
};
use worth_store::physical_runtime::production::PhysicalMutationCheckpoint;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, ServingPhysicalRuntime,
};

pub(super) fn prepare_for_checkpoint(
    serving: &ServingPhysicalRuntime,
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    seed: u64,
    invocation: &Invocation,
) -> Result<impl Sized, String> {
    let gate = serving.pause_physical_mutation_at(
        PhysicalMutationCheckpoint::AfterWritebackAdmissionBeforeEffect,
    );
    let mutation = start_dirty_checkpoint(
        serving,
        placement,
        dirty_material(seed),
        configuration::dirty_checkpoint_payload_length(format),
    )?;
    if !gate.await_arrival() {
        gate.release();
        return Err(
            "ordinary C8 dirty mutation did not reach after-writeback-admission-before-effect"
                .to_owned(),
        );
    }
    markers::write_ready(&invocation.start_marker, "write C8 writer ready marker")?;
    markers::wait_for_parent(&invocation.start_marker);
    Ok((gate, mutation))
}
